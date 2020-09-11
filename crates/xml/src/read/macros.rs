/// macro to require specified byte is read next
#[macro_export]
macro_rules! require_byte {
    ($b:literal, $self:ident, $err:path) => {
        if !$self.next_byte_is($b)? {
            return Err($err);
        }
    };
}

/// macro to require specified string is read next
#[macro_export]
macro_rules! require_str {
    ($s:literal, $self:ident, $err:path) => {
        unsafe {
            if !$self.expect_str($s)? {
                return Err($err);
            }
        }
    };
}

// macro to require at least one space character read next
#[macro_export]
macro_rules! require_whitespace {
    ($self:ident, $err:path) => {
        if !$self.whitespace()? {
            return Err($err);
        }
    };
}
