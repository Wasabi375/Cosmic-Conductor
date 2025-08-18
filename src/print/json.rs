use std::{fmt::Write as _, io::Write, marker::PhantomData};

use super::{Print, PrintList, SaveDrop};

use anyhow::Result;

pub struct Printer<'a, W: Write> {
    buffer: &'a mut String,
    first: bool,
    dropped: bool,
    _phantom: PhantomData<W>,
}

impl<'a, W: Write> Printer<'a, W> {
    pub fn new<'n>(buffer: &'n mut String) -> Result<Printer<'n, W>> {
        write!(buffer, "{{")?;
        Ok(Printer {
            buffer,
            first: true,
            dropped: false,
            _phantom: Default::default(),
        })
    }

    fn field(&mut self, name: &str) -> Result<()> {
        self.comma()?;
        write!(self.buffer, "\"{name}\":")?;
        Ok(())
    }

    fn comma(&mut self) -> Result<()> {
        if !self.first {
            write!(self.buffer, ",")?;
        }
        self.first = false;
        Ok(())
    }
}

impl<W: Write> SaveDrop for Printer<'_, W> {
    fn save_drop(&mut self) -> Result<()> {
        if !self.dropped {
            self.dropped = true;
            write!(self.buffer, "}}")?;
        }
        Ok(())
    }
}

impl<W: Write> Print<W> for Printer<'_, W> {
    fn field<D: std::fmt::Display>(&mut self, name: &str, value: D) -> Result<()> {
        self.field(name)?;
        write!(self.buffer, "\"{value}\"")?;
        Ok(())
    }

    fn sub_struct<'a>(&'a mut self, name: &str) -> Result<super::Printer<'a, W>> {
        self.field(name)?;
        Ok(Printer::new(self.buffer)?.into())
    }

    fn sub_list_with(
        &mut self,
        name: &str,
        _options: super::ListOptions,
    ) -> Result<super::ListPrinter<W>> {
        self.field(name)?;
        Ok(ListPrinter::new(self.buffer)?.into())
    }
}

impl<W: Write> Drop for Printer<'_, W> {
    fn drop(&mut self) {
        self.save_drop().unwrap()
    }
}

pub struct ListPrinter<'a, W: Write> {
    buffer: &'a mut String,
    first: bool,
    dropped: bool,
    _phantom: PhantomData<W>,
}

impl<W: Write> ListPrinter<'_, W> {
    fn new(buffer: &mut String) -> Result<ListPrinter<'_, W>> {
        write!(buffer, "[")?;
        Ok(ListPrinter {
            buffer,
            first: true,
            dropped: false,
            _phantom: Default::default(),
        })
    }

    fn comma(&mut self) -> Result<()> {
        if !self.first {
            write!(self.buffer, ",")?;
        }
        self.first = false;
        Ok(())
    }
}

impl<W: Write> PrintList<W> for ListPrinter<'_, W> {
    fn item<D: std::fmt::Display>(&mut self, value: D) -> Result<()> {
        self.comma()?;
        write!(self.buffer, "\"{value}\"")?;
        Ok(())
    }

    fn sub_struct<'n>(&'n mut self) -> Result<super::Printer<'n, W>> {
        self.comma()?;
        Ok(Printer::new(self.buffer)?.into())
    }

    fn sub_list_with<'n>(
        &'n mut self,
        _options: super::ListOptions,
    ) -> Result<super::ListPrinter<'n, W>> {
        self.comma()?;
        Ok(ListPrinter::new(self.buffer)?.into())
    }
}

impl<W: Write> SaveDrop for ListPrinter<'_, W> {
    fn save_drop(&mut self) -> Result<()> {
        if !self.dropped {
            self.dropped = true;
            write!(self.buffer, "]")?;
        }
        Ok(())
    }
}

impl<W: Write> Drop for ListPrinter<'_, W> {
    fn drop(&mut self) {
        self.save_drop().unwrap();
    }
}
