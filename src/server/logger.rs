extern crate slog;
extern crate thread_id;

use std::result;
use slog::*;

thread_local!(static TL_THREAD_ID: usize = thread_id::get() % 65536);

#[derive(Clone)]
pub struct ThreadLocalDrain<D>
    where
        D: Drain
{
    pub drain: D,
}

impl<D> Drain for ThreadLocalDrain<D>
    where D: Drain
{
    type Ok = ();
    type Err = Never;

    fn log(
        &self,
        record: &Record,
        values: &OwnedKVList,
    ) -> result::Result<Self::Ok, Self::Err> {
        let chained = OwnedKVList::from(OwnedKV((SingleKV("thread",
                                                          TL_THREAD_ID.with(|id| { *id }),
        ), values.clone())));
        let _ = self.drain.log(record, &chained);
        Ok(())
    }
}


pub struct FnGuard {
    function_name: &'static str,
    logger: Logger,
}

impl FnGuard {
    pub fn new<T>(logger: Logger, values: OwnedKV<T>, function_name: &'static str) -> FnGuard
        where
            T: SendSyncRefUnwindSafeKV + 'static
    {
        let new_logger = logger.new(values);
        info!(new_logger, "[Enter] {}", function_name);
        FnGuard {
            function_name,
            logger: new_logger,
        }
    }

    pub fn log(&self, record: &Record) {
        self.logger.log(record);
    }
}

impl Drop for FnGuard {
    fn drop(&mut self) {
        info!(self.logger, "[Exit] {}", self.function_name);
    }
}