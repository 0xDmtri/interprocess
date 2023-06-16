//! The limbo which dropped streams are sent to if send buffer preservation is enabled.
//!
//! Because dropping a named pipe file handle, be it a client or a server, discards its send buffer, the portability-conscious local socket interface requires this additional feature to allow for the common use case of dropping right after sending a graceful shutdown message.

const LIMBO_SLOTS: usize = 16;

/// Common result type for operations that complete with no output but may reject their input, requiring some form of retry.
pub(crate) type MaybeReject<T> = Result<(), T>;

pub(crate) struct LimboPool<S> {
    senders: [Option<S>; LIMBO_SLOTS],
    count: usize,
    count_including_overflow: usize,
}
impl<S> LimboPool<S> {
    pub fn add_sender(&mut self, s: S) -> MaybeReject<S> {
        self.count_including_overflow += 1;
        if self.count < LIMBO_SLOTS {
            self.senders[self.count] = Some(s);
            self.count += 1;
            Ok(())
        } else {
            Err(s)
        }
    }
    /// Tries shoving the given accumulant into the given maybe-rejecting function with every available sender.
    pub fn linear_try<T>(&mut self, acc: T, mut f: impl FnMut(&mut S, T) -> MaybeReject<T>) -> MaybeReject<T> {
        let mut acc = Some(acc);
        for sender in &mut self.senders[0..self.count] {
            let regain = match f(sender.as_mut().unwrap(), acc.take().unwrap()) {
                Ok(()) => return Ok(()),
                Err(r) => r,
            };
            acc = Some(regain);
        }
        Err(acc.unwrap())
    }
    /// Performs `linear_try` with `acc` and `tryf`, and if that fails, calls `createf` and consumes its output with `add_sender` if the pool has vacant space, resorting to `fullf` otherwise.
    pub fn linear_try_or_create<T>(
        &mut self,
        acc: T,
        tryf: impl FnMut(&mut S, T) -> MaybeReject<T>,
        // First argument is the index of the new sender.
        createf: impl FnOnce(usize, T) -> S,
        // Same here.
        fullf: impl FnOnce(usize, T),
    ) {
        let acc = match self.linear_try(acc, tryf) {
            Ok(()) => return,
            Err(regain) => regain,
        };
        if self.count < LIMBO_SLOTS {
            // Cannot error.
            let _ = self.add_sender(createf(self.count, acc));
        } else {
            fullf(self.count_including_overflow, acc);
            self.count_including_overflow += 1;
        }
    }
}
impl<S> Default for LimboPool<S> {
    fn default() -> Self {
        Self {
            // hmm today i will initialize an array
            senders: Default::default(),
            count: 0,
            count_including_overflow: 0,
        }
    }
}
