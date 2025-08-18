use anyhow::Result;
use std::{
    fmt::{Debug, Display},
    io::Write,
};

pub mod human;
pub mod json;

pub fn human<'a, W: Write + 'a>(writer: &'a mut W) -> Printer<'a, W> {
    Printer::Human(human::Printer::new(writer))
}

pub fn json<'a, W: Write + 'a>(buffer: &'a mut String) -> Result<Printer<'a, W>> {
    Ok(Printer::<W>::Json(json::Printer::new(buffer)?))
}

pub trait SaveDrop {
    /// Allows for catching errors that happen during drop.
    ///
    /// This takes a `&mut self` just like drop.
    /// Using `self` after a call to `save_drop` is undefined behaviour
    /// in terms of what it does but it will not break any of rusts guarantees.
    fn save_drop(&mut self) -> Result<()>;
}

#[derive(Debug, Clone, Default)]
pub struct ListOptions {
    pub inline: bool,
}

pub trait Print<W: Write>: SaveDrop {
    fn field<D: Display>(&mut self, name: &str, value: D) -> Result<()>;

    #[allow(dead_code)]
    fn sub_struct<'n>(&'n mut self, name: &str) -> Result<Printer<'n, W>>;
    fn sub_list_with<'n>(
        &'n mut self,
        name: &str,
        options: ListOptions,
    ) -> Result<ListPrinter<'n, W>>;

    fn sub_list<'n>(&'n mut self, name: &str) -> Result<ListPrinter<'n, W>> {
        self.sub_list_with(name, ListOptions::default())
    }

    fn optional<D: Display>(&mut self, name: &str, value: Option<D>) -> Result<()> {
        if let Some(value) = value {
            self.field(name, value)
        } else {
            Ok(())
        }
    }

    fn inline_list<D: Display>(&mut self, name: &str, list: impl Iterator<Item = D>) -> Result<()> {
        self.list_with(name, list, ListOptions { inline: true })
    }

    #[allow(dead_code)]
    fn list<D: Display>(&mut self, name: &str, list: impl Iterator<Item = D>) -> Result<()> {
        self.list_with(name, list, ListOptions::default())
    }

    fn list_with<D: Display>(
        &mut self,
        name: &str,
        list: impl Iterator<Item = D>,
        options: ListOptions,
    ) -> Result<()> {
        let mut list_printer = self.sub_list_with(name, options)?;

        for item in list {
            list_printer.item(item)?;
        }

        list_printer.save_drop()?;

        Ok(())
    }

    fn field_debug<D: Debug>(&mut self, name: &str, value: D) -> Result<()> {
        self.field(name, DebugToDisplay(value))
    }
}

pub trait PrintList<W: Write>: SaveDrop {
    fn item<D: Display>(&mut self, value: D) -> Result<()>;

    fn sub_struct<'n>(&'n mut self) -> Result<Printer<'n, W>>;
    fn sub_list_with<'n>(&'n mut self, options: ListOptions) -> Result<ListPrinter<'n, W>>;

    #[allow(dead_code)]
    fn sub_list<'n>(&'n mut self) -> Result<ListPrinter<'n, W>> {
        self.sub_list_with(ListOptions::default())
    }

    #[allow(dead_code)]
    fn optional<D: Display>(&mut self, value: Option<D>) -> Result<()> {
        if let Some(value) = value {
            self.item(value)
        } else {
            Ok(())
        }
    }

    #[allow(dead_code)]
    fn list<D: Display>(&mut self, list: impl Iterator<Item = D>) -> Result<()> {
        let mut list_printer = self.sub_list()?;

        for item in list {
            list_printer.item(item)?;
        }

        list_printer.save_drop()?;

        Ok(())
    }

    #[allow(dead_code)]
    fn item_debug<D: Debug>(&mut self, value: D) -> Result<()> {
        self.item(DebugToDisplay(value))
    }
}

pub enum Printer<'a, W: Write> {
    Human(human::Printer<'a, W>),
    Json(json::Printer<'a, W>),
}

impl<'a, W: Write> From<human::Printer<'a, W>> for Printer<'a, W> {
    fn from(value: human::Printer<'a, W>) -> Self {
        Self::Human(value)
    }
}

impl<'a, W: Write> From<json::Printer<'a, W>> for Printer<'a, W> {
    fn from(value: json::Printer<'a, W>) -> Self {
        Self::Json(value)
    }
}

impl<W: Write> SaveDrop for Printer<'_, W> {
    fn save_drop(&mut self) -> Result<()> {
        match self {
            Printer::Human(printer) => printer.save_drop(),
            Printer::Json(printer) => printer.save_drop(),
        }
    }
}

impl<W: Write> Print<W> for Printer<'_, W> {
    fn field<D: Display>(&mut self, name: &str, value: D) -> Result<()> {
        match self {
            Printer::Human(printer) => printer.field(name, value),
            Printer::Json(printer) => printer.field(name, value),
        }
    }

    fn sub_struct(&mut self, name: &str) -> Result<Printer<W>> {
        match self {
            Printer::Human(printer) => printer.sub_struct(name),
            Printer::Json(printer) => printer.sub_struct(name),
        }
    }

    fn sub_list_with(&mut self, name: &str, options: ListOptions) -> Result<ListPrinter<W>> {
        match self {
            Printer::Human(printer) => printer.sub_list_with(name, options),
            Printer::Json(printer) => printer.sub_list_with(name, options),
        }
    }
}

impl<W: Write> Drop for Printer<'_, W> {
    fn drop(&mut self) {
        self.save_drop().unwrap();
    }
}

pub enum ListPrinter<'a, W: Write> {
    Human(human::ListPrinter<'a, W>),
    Json(json::ListPrinter<'a, W>),
}

impl<'a, W: Write> From<human::ListPrinter<'a, W>> for ListPrinter<'a, W> {
    fn from(value: human::ListPrinter<'a, W>) -> Self {
        Self::Human(value)
    }
}

impl<'a, W: Write> From<json::ListPrinter<'a, W>> for ListPrinter<'a, W> {
    fn from(value: json::ListPrinter<'a, W>) -> Self {
        Self::Json(value)
    }
}

impl<W: Write> SaveDrop for ListPrinter<'_, W> {
    fn save_drop(&mut self) -> Result<()> {
        match self {
            ListPrinter::Human(printer) => printer.save_drop(),
            ListPrinter::Json(printer) => printer.save_drop(),
        }
    }
}

impl<W: Write> PrintList<W> for ListPrinter<'_, W> {
    fn item<D: Display>(&mut self, value: D) -> Result<()> {
        match self {
            ListPrinter::Human(list_printer) => list_printer.item(value),
            ListPrinter::Json(list_printer) => list_printer.item(value),
        }
    }

    fn sub_struct(&mut self) -> Result<Printer<W>> {
        match self {
            ListPrinter::Human(list_printer) => list_printer.sub_struct(),
            ListPrinter::Json(list_printer) => list_printer.sub_struct(),
        }
    }

    fn sub_list_with(&mut self, options: ListOptions) -> Result<ListPrinter<W>> {
        match self {
            ListPrinter::Human(list_printer) => list_printer.sub_list_with(options),
            ListPrinter::Json(list_printer) => list_printer.sub_list_with(options),
        }
    }
}

impl<W: Write> Drop for ListPrinter<'_, W> {
    fn drop(&mut self) {
        self.save_drop().unwrap();
    }
}

struct DebugToDisplay<T>(T);

impl<T: Debug> Display for DebugToDisplay<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.0))
    }
}
