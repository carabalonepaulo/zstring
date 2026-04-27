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

    pub fn pop(&mut self) -> Option<char> {
        let ch = self.as_str().chars().last()?;
        let new_len = self.len() - ch.len_utf8();
        self.truncate(new_len);
        Some(ch)
    }

    pub fn insert(&mut self, index: usize, ch: char) {
        assert!(self.as_str().is_char_boundary(index));

        let char_len = ch.len_utf8();
        let mut char_buf = [0u8; 4];
        ch.encode_utf8(&mut char_buf);
        let char_bytes = &char_buf[..char_len];

        match self {
            Self::Inline { data, len } => {
                let current_len = *len as usize;
                if current_len + char_len <= 29 {
                    unsafe {
                        let ptr = data.as_mut_ptr().add(index);
                        std::ptr::copy(ptr, ptr.add(char_len), current_len - index);
                        std::ptr::copy_nonoverlapping(char_bytes.as_ptr(), ptr, char_len);
                    }
                    *len = (current_len + char_len) as u8;
                    data[*len as usize] = 0;
                } else {
                    let mut vec = Vec::with_capacity(current_len + char_len + 1);
                    vec.extend_from_slice(&data[..index]);
                    vec.extend_from_slice(char_bytes);
                    vec.extend_from_slice(&data[index..current_len]);
                    vec.push(0);
                    *self = Self::Heap(vec);
                }
            }
            Self::Heap(vec) => {
                vec.pop();
                for (i, &byte) in char_bytes.iter().enumerate() {
                    vec.insert(index + i, byte);
                }
                vec.push(0);
            }
        }
    }

    pub fn clear(&mut self) {
        match self {
            Self::Inline { data, len } => {
                *len = 0;
                data[0] = 0;
            }
            Self::Heap(vec) => {
                vec.clear();
                vec.push(0);
            }
        }
    }

    pub fn truncate(&mut self, new_len: usize) {
        if new_len > self.len() {
            return;
        }

        match self {
            Self::Inline { data, len } => {
                *len = new_len as u8;
                data[new_len] = 0;
            }
            Self::Heap(vec) => {
                vec.truncate(new_len);
                vec.push(0);
            }
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::Inline { len, .. } => *len as usize,
            Self::Heap(vec) => vec.len() - 1,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn capacity(&self) -> usize {
        match self {
            Self::Inline { .. } => 29,
            Self::Heap(vec) => vec.capacity() - 1,
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

    pub fn into_string(self) -> String {
        match self {
            Self::Inline { data, len } => unsafe {
                std::str::from_utf8_unchecked(&data[..len as usize]).to_string()
            },
            Self::Heap(mut vec) => unsafe {
                vec.pop();
                String::from_utf8_unchecked(vec)
            },
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

impl From<&str> for ZString {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for ZString {
    fn from(s: String) -> Self {
        Self::new(s)
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

    #[test]
    fn pop_char_inline() {
        let mut s = ZString::new("Rúst");
        let ch = s.pop();
        assert_eq!(ch, Some('t'));
        assert_eq!(s.as_str(), "Rús");
        assert_eq!(s.len(), 4);
        unsafe {
            assert_eq!(*s.as_ptr().add(4), 0);
        }
    }

    #[test]
    fn pop_char_heap() {
        let mut s = ZString::new("a".repeat(30));
        let ch = s.pop();
        assert_eq!(ch, Some('a'));
        assert_eq!(s.len(), 29);
        assert!(matches!(s, ZString::Heap(_)));
        unsafe {
            assert_eq!(*s.as_ptr().add(29), 0);
        }
    }

    #[test]
    fn clear_inline() {
        let mut s = ZString::new("content");
        s.clear();
        assert_eq!(s.len(), 0);
        assert!(s.is_empty());
        unsafe {
            assert_eq!(*s.as_ptr(), 0);
        }
    }

    #[test]
    fn clear_heap() {
        let mut s = ZString::new("a".repeat(40));
        s.clear();
        assert_eq!(s.len(), 0);
        assert!(matches!(s, ZString::Heap(_)));
        unsafe {
            assert_eq!(*s.as_ptr(), 0);
        }
    }

    #[test]
    fn truncate_inline() {
        let mut s = ZString::new("0123456789");
        s.truncate(5);
        assert_eq!(s.as_str(), "01234");
        unsafe {
            assert_eq!(*s.as_ptr().add(5), 0);
        }
    }

    #[test]
    fn truncate_longer_than_len() {
        let mut s = ZString::new("abc");
        s.truncate(10);
        assert_eq!(s.as_str(), "abc");
    }

    #[test]
    fn into_string_inline() {
        let s = ZString::new("inline_test");
        let std_s = s.into_string();
        assert_eq!(std_s, "inline_test");
    }

    #[test]
    fn into_string_heap() {
        let input = "a".repeat(35);
        let s = ZString::new(&input);
        let std_s = s.into_string();
        assert_eq!(std_s, input);
    }

    #[test]
    fn capacity_check() {
        let s_inline = ZString::new("short");
        assert_eq!(s_inline.capacity(), 29);

        let s_heap = ZString::new("a".repeat(50));
        assert!(s_heap.capacity() >= 50);
    }

    #[test]
    fn empty_pop() {
        let mut s = ZString::new("");
        assert_eq!(s.pop(), None);
    }

    #[test]
    fn insert_at_start_inline() {
        let mut s = ZString::new("world");
        s.insert(0, 'h');
        assert_eq!(s.as_str(), "hworld");
        unsafe {
            assert_eq!(*s.as_ptr().add(6), 0);
        }
    }

    #[test]
    fn insert_in_middle_inline() {
        let mut s = ZString::new("ab");
        s.insert(1, '!');
        assert_eq!(s.as_str(), "a!b");
        unsafe {
            assert_eq!(*s.as_ptr().add(0), b'a' as _);
            assert_eq!(*s.as_ptr().add(1), b'!' as _);
            assert_eq!(*s.as_ptr().add(2), b'b' as _);
            assert_eq!(*s.as_ptr().add(3), 0);
        }
    }

    #[test]
    fn insert_multi_byte_char_inline() {
        let mut s = ZString::new("rust");
        s.insert(0, '🦀');
        assert_eq!(s.as_str(), "🦀rust");
        assert_eq!(s.len(), 8);
        unsafe {
            assert_eq!(*s.as_ptr().add(8), 0);
        }
    }

    #[test]
    fn insert_trigger_promotion() {
        let mut s = ZString::new("a".repeat(26));
        s.insert(10, '🦀');

        assert!(matches!(s, ZString::Heap(_)));
        assert_eq!(s.len(), 30);
        assert!(s.as_str().contains('🦀'));
        unsafe {
            assert_eq!(*s.as_ptr().add(30), 0);
        }
    }

    #[test]
    fn insert_heap_boundary() {
        let mut s = ZString::new("a".repeat(30));
        s.insert(30, '!');
        assert_eq!(s.len(), 31);
        assert_eq!(s.as_str().chars().last(), Some('!'));
        unsafe {
            assert_eq!(*s.as_ptr().add(31), 0);
        }
    }

    #[test]
    #[should_panic]
    fn insert_invalid_boundary_panic() {
        let mut s = ZString::new("á");
        s.insert(1, '!');
    }
}
