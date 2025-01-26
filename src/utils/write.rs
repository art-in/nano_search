use std::fmt::Display;
use std::io::Write;

pub trait Print {
    fn print<D: Display>(&mut self, string: D);
    fn println<D: Display>(&mut self, string: D);
}

impl<W: ?Sized + std::io::Write> Print for std::io::BufWriter<W> {
    fn print<D: Display>(&mut self, string: D) {
        write!(self, "{string}")
            .expect("string should be written to output stream");
    }
    fn println<D: Display>(&mut self, string: D) {
        write!(self, "{string}")
            .expect("string should be written to output stream");
        writeln!(self).expect("new line should be written to output stream");
    }
}
