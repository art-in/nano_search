use std::io::Write;

/// Decorator for [`std::io::Write`] that counts number of written bytes.
pub struct CountingWriter<W> {
    inner: W,
    written_bytes: usize,
}

impl<W: Write> CountingWriter<W> {
    pub const fn new(inner: W) -> Self {
        Self {
            inner,
            written_bytes: 0,
        }
    }

    #[inline]
    pub const fn get_written_bytes(&self) -> usize {
        self.written_bytes
    }

    #[inline]
    pub fn into_inner(self) -> W {
        self.inner
    }
}

impl<W: Write> Write for CountingWriter<W> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let written = self.inner.write(buf)?;
        self.written_bytes += written;
        Ok(written)
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.inner.write_all(buf)?;
        self.written_bytes += buf.len();
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()?;
        Ok(())
    }
}
