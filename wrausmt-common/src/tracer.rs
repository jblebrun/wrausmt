/// A tracer tracks some context strings, providing a way for them to be
/// automatically dropped based on scoping.
pub struct Tracer {
    msgs: Vec<String>,
}

/// A [TraceDropper] is returned from a call to [Tracer::trace]. You should hold
/// a reference to it for as long as you want the traced message to stay in the
/// list of messages.
pub struct TraceDropper {
    /// The message we expect to drop.
    msg:  String,
    /// A mutable pointer to the msg list containing the msg.
    msgs: *mut Vec<String>,
}

impl Tracer {
    /// Create a new [Tracer] instance with the initial string.
    pub fn new(msg: &str) -> Tracer {
        Tracer {
            msgs: vec![msg.into()],
        }
    }

    /// Add the provided `msg` to the trace stack. The msg will be removed when
    /// [TraceDropper] goes out of scope.
    ///
    /// This is a prototype library. If the [TraceDropper] detects a mismatch
    /// between what it expects to drop and what it will actually drop, it will
    /// panic.
    pub fn trace(&mut self, msg: &str) -> TraceDropper {
        self.msgs.push(msg.into());
        TraceDropper {
            msg:  msg.into(),
            msgs: &mut self.msgs as *mut Vec<String>,
        }
    }

    /// Return a clone of all of the currently stored messages.
    pub fn clone_msgs(&self) -> Vec<String> {
        self.msgs.clone()
    }
}

impl Drop for TraceDropper {
    /// Drops the message that was pushed when this [TraceDropper] was created.
    fn drop(&mut self) {
        let msgs = unsafe { &mut *self.msgs };
        if Some(&self.msg) != msgs.last() {
            panic!("msgs mismtach: {} {:?}", self.msg, msgs);
        }
        let _x = msgs.pop();
    }
}
