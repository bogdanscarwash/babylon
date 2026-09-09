//! Shared bounded cursor for material-circuit wire versions.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CursorError {
    Truncated,
    Trailing,
}

pub(crate) struct Cursor<'a> {
    bytes: &'a [u8],
    index: usize,
}

impl<'a> Cursor<'a> {
    pub(crate) const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, index: 0 }
    }

    pub(crate) fn take(&mut self, length: usize) -> Result<&'a [u8], CursorError> {
        let end = self
            .index
            .checked_add(length)
            .ok_or(CursorError::Truncated)?;
        let output = self
            .bytes
            .get(self.index..end)
            .ok_or(CursorError::Truncated)?;
        self.index = end;
        Ok(output)
    }

    pub(crate) fn array<const N: usize>(&mut self) -> Result<[u8; N], CursorError> {
        self.take(N)?.try_into().map_err(|_| CursorError::Truncated)
    }

    pub(crate) fn u8(&mut self) -> Result<u8, CursorError> {
        Ok(self.array::<1>()?[0])
    }

    pub(crate) fn u16(&mut self) -> Result<u16, CursorError> {
        Ok(u16::from_be_bytes(self.array()?))
    }

    pub(crate) fn u32(&mut self) -> Result<u32, CursorError> {
        Ok(u32::from_be_bytes(self.array()?))
    }

    pub(crate) fn u64(&mut self) -> Result<u64, CursorError> {
        Ok(u64::from_be_bytes(self.array()?))
    }

    pub(crate) fn finish(self) -> Result<(), CursorError> {
        if self.index == self.bytes.len() {
            Ok(())
        } else {
            Err(CursorError::Trailing)
        }
    }
}
