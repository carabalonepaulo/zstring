#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ZString {
    Inline { data: [u8; 30], len: u8 },
    Heap(Vec<u8>),
}

impl ZString {
    pub fn new(value: impl AsRef<str>) -> Self {
        let buf = value.as_ref().as_bytes();
        if buf.len() <= 29 {
            let mut data = [0u8; 30];
            data[..buf.len()].copy_from_slice(buf);
            Self::Inline {
                data,
                len: buf.len() as u8,
            }
        } else {
            let mut vec = Vec::with_capacity(buf.len() + 1);
            vec.extend_from_slice(buf);
            vec.push(0);
            Self::Heap(vec)
        }
    }

    pub fn set(&mut self, value: impl AsRef<str>) {
        let buf = value.as_ref().as_bytes();
        let new_len = buf.len();

        match self {
            Self::Inline { data, len } => {
                if new_len <= 29 {
                    data[..new_len].copy_from_slice(buf);
                    data[new_len] = 0;
                    *len = new_len as u8;
                } else {
                    let mut vec = Vec::with_capacity(new_len + 1);
                    vec.extend_from_slice(buf);
                    vec.push(0);
                    *self = Self::Heap(vec);
                }
            }
            Self::Heap(vec) => {
                vec.clear();
                vec.extend_from_slice(buf);
                vec.push(0);
            }
        }
    }

    pub fn push(&mut self, value: impl AsRef<str>) {
        let extra = value.as_ref().as_bytes();
        if extra.is_empty() {
            return;
        }

        match self {
            Self::Inline { data, len } => {
                let new_len = *len as usize + extra.len();
                if new_len <= 29 {
                    data[*len as usize..new_len].copy_from_slice(extra);
                    data[new_len] = 0;
                    *len = new_len as u8;
                } else {
                    let mut vec = Vec::with_capacity(new_len + 1);
                    vec.extend_from_slice(&data[..*len as usize]);
                    vec.extend_from_slice(extra);
                    vec.push(0);
                    *self = Self::Heap(vec);
                }
            }
            Self::Heap(vec) => {
                vec.pop();
                vec.extend_from_slice(extra);
                vec.push(0);
            }
        }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        let buf = match self {
            Self::Inline { data, len } => &data[..*len as usize],
            Self::Heap(buf) => &buf[..buf.len() - 1],
        };
        unsafe { std::str::from_utf8_unchecked(buf) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const i8 {
        match self {
            Self::Inline { data, .. } => data.as_ptr() as *const i8,
            Self::Heap(buf) => buf.as_ptr() as *const i8,
        }
    }

    #[inline]
    pub fn parts(&self) -> (*const u8, usize) {
        match self {
            Self::Inline { data, len } => (data.as_ptr(), *len as usize),
            Self::Heap(buf) => (buf.as_ptr(), buf.len() - 1),
        }
    }
}

impl AsRef<str> for ZString {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for ZString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for ZString {
    fn default() -> Self {
        Self::Inline {
            data: [0; 30],
            len: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_creation() {
        let s = ZString::new("Hello");
        assert!(matches!(s, ZString::Inline { len: 5, .. }));
        assert_eq!(s.as_str(), "Hello");
        unsafe {
            assert_eq!(*s.as_ptr().add(5), 0);
        }
    }

    #[test]
    fn heap_promotion() {
        let input = "a".repeat(30);
        let s = ZString::new(&input);
        assert!(matches!(s, ZString::Heap(_)));
        assert_eq!(s.as_str(), input);
        unsafe {
            assert_eq!(*s.as_ptr().add(30), 0);
        }
    }

    #[test]
    fn set_sticky_heap() {
        let mut s = ZString::new("a".repeat(35));
        let ptr_before = s.as_ptr();

        s.set("short");
        let ptr_after = s.as_ptr();

        assert!(matches!(s, ZString::Heap(_)));
        assert_eq!(ptr_before, ptr_after);
        assert_eq!(s.as_str(), "short");
        unsafe {
            assert_eq!(*s.as_ptr().add(5), 0);
        }
    }

    #[test]
    fn push_inline_to_heap() {
        let mut s = ZString::new("abc");
        s.push("d".repeat(30));

        assert!(matches!(s, ZString::Heap(_)));
        assert_eq!(s.as_str().len(), 33);
        unsafe {
            assert_eq!(*s.as_ptr().add(33), 0);
        }
    }

    #[test]
    fn push_heap_logic() {
        let mut s = ZString::new("a".repeat(30));
        s.push("b");

        assert_eq!(s.as_str().len(), 31);
        assert_eq!(&s.as_str()[30..], "b");
        unsafe {
            assert_eq!(*s.as_ptr().add(31), 0);
        }
    }

    #[test]
    fn parts_consistency() {
        let s_inline = ZString::new("inline");
        let (_, len_i) = s_inline.parts();
        assert_eq!(len_i, 6);

        let s_heap = ZString::new("a".repeat(40));
        let (_, len_h) = s_heap.parts();
        assert_eq!(len_h, 40);
    }

    #[test]
    fn default_state() {
        let s = ZString::default();
        assert_eq!(s.as_str(), "");
        unsafe {
            assert_eq!(*s.as_ptr(), 0);
        }
    }

    #[test]
    fn null_terminator_on_set() {
        let mut s = ZString::new("original");
        s.set("new");
        unsafe {
            assert_eq!(*s.as_ptr().add(3), 0);
        }
    }
}
