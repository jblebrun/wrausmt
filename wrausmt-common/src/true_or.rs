pub trait TrueOr {
    fn true_or<E>(&self, err: E) -> std::result::Result<(), E>;
    fn true_or_else<E, F: Fn() -> E>(&self, err: F) -> std::result::Result<(), E>;
}

impl TrueOr for bool {
    fn true_or_else<E, F: Fn() -> E>(&self, err: F) -> std::result::Result<(), E> {
        if *self {
            Ok(())
        } else {
            Err(err())
        }
    }

    fn true_or<E>(&self, err: E) -> std::result::Result<(), E> {
        if *self {
            Ok(())
        } else {
            Err(err)
        }
    }
}
