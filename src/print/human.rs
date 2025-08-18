use std::io::Write;

use anyhow::{Result, ensure};

use super::{ListOptions, Print, PrintList, SaveDrop};

pub const INDENT: &str = "  ";

pub struct Printer<'w, W: Write> {
    writer: &'w mut W,
    end_with_nl: bool,
    indent: String,
}

impl<'w, W: Write> Printer<'w, W> {
    pub fn new(writer: &'w mut W) -> Self {
        Self {
            writer,
            end_with_nl: false,
            indent: "".into(),
        }
    }
}

impl<'w, W: Write> SaveDrop for Printer<'w, W> {
    fn save_drop(&mut self) -> Result<()> {
        if self.end_with_nl {
            self.end_with_nl = false;
            writeln!(self.writer)?;
        }
        Ok(())
    }
}

impl<'w, W: Write> Print for Printer<'w, W> {
    fn field<D: std::fmt::Display>(&mut self, name: &str, value: D) -> Result<()> {
        writeln!(self.writer, "{}{name}: {value}", self.indent)?;
        Ok(())
    }

    fn sub_struct(&mut self, name: &str) -> Result<impl Print> {
        writeln!(self.writer, "{}{name}", self.indent)?;
        Ok(Printer {
            writer: self.writer,
            end_with_nl: true,
            indent: format!("{INDENT}{}", self.indent),
        })
    }

    fn sub_list_with(&mut self, name: &str, options: ListOptions) -> Result<impl PrintList> {
        if options.inline {
            write!(self.writer, "{}{name}: ", self.indent)?;
        } else {
            writeln!(self.writer, "{}{name}:", self.indent)?;
        }
        Ok(ListPrinter::new(
            self.writer,
            format!("{INDENT}{}", self.indent),
            options,
        ))
    }
}

impl<'w, W: Write> Drop for Printer<'w, W> {
    fn drop(&mut self) {
        self.save_drop().unwrap();
    }
}

pub struct ListPrinter<'w, W: Write> {
    writer: &'w mut W,
    counter: u32,
    end_with_nl: bool,
    indent: String,
    options: ListOptions,
}

impl<'w, W: Write> ListPrinter<'w, W> {
    pub fn new(writer: &'w mut W, indent: String, options: ListOptions) -> Self {
        Self {
            writer,
            counter: 0,
            end_with_nl: true,
            indent,
            options,
        }
    }
}

impl<'w, W: Write> SaveDrop for ListPrinter<'w, W> {
    fn save_drop(&mut self) -> Result<()> {
        if self.end_with_nl {
            self.end_with_nl = false;
            writeln!(self.writer)?;
        }
        Ok(())
    }
}

impl<'w, W: Write> PrintList for ListPrinter<'w, W> {
    fn item<D: std::fmt::Display>(&mut self, value: D) -> Result<()> {
        self.counter += 1;
        if self.options.inline {
            if self.counter > 1 {
                write!(self.writer, ", ")?;
            }
            write!(self.writer, "{value}")?;
        } else {
            writeln!(self.writer, "{}{}: {value}", self.indent, self.counter)?;
        }
        Ok(())
    }

    fn sub_struct(&mut self) -> Result<impl Print> {
        ensure!(
            !self.options.inline,
            "Inline list does not support inner struct"
        );
        self.counter += 1;
        writeln!(self.writer, "{}{}:", self.indent, self.counter)?;
        Ok(Printer {
            writer: self.writer,
            end_with_nl: true,
            indent: self.indent.clone(),
        })
    }

    fn sub_list_with(&mut self, options: ListOptions) -> Result<impl PrintList> {
        ensure!(
            !self.options.inline,
            "Inline list does not support inner list"
        );
        self.counter += 1;
        if options.inline {
        } else {
            writeln!(self.writer, "{}{}:", self.indent, self.counter)?;
        }
        Ok(ListPrinter {
            writer: self.writer,
            counter: 0,
            end_with_nl: true,
            indent: format!("{INDENT}{}", self.indent),
            options,
        })
    }
}

impl<'w, W: Write> Drop for ListPrinter<'w, W> {
    fn drop(&mut self) {
        self.save_drop().unwrap();
    }
}
