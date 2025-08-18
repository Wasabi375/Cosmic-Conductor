use anyhow::Result;
use std::fmt::{Debug, Display, Write};

pub mod human;
// pub mod json;

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

pub trait Print: SaveDrop {
    fn field<D: Display>(&mut self, name: &str, value: D) -> Result<()>;

    fn sub_struct(&mut self, name: &str) -> Result<impl Print>;
    fn sub_list_with(&mut self, name: &str, options: ListOptions) -> Result<impl PrintList>;

    fn sub_list(&mut self, name: &str) -> Result<impl PrintList> {
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

pub trait PrintList: SaveDrop {
    fn item<D: Display>(&mut self, value: D) -> Result<()>;

    fn sub_struct(&mut self) -> Result<impl Print>;
    fn sub_list_with(&mut self, options: ListOptions) -> Result<impl PrintList>;

    fn sub_list(&mut self) -> Result<impl PrintList> {
        self.sub_list_with(ListOptions::default())
    }
    fn optional<D: Display>(&mut self, value: Option<D>) -> Result<()> {
        if let Some(value) = value {
            self.item(value)
        } else {
            Ok(())
        }
    }

    fn list<D: Display>(&mut self, list: impl Iterator<Item = D>) -> Result<()> {
        let mut list_printer = self.sub_list()?;

        for item in list {
            list_printer.item(item)?;
        }

        list_printer.save_drop()?;

        Ok(())
    }

    fn item_debug<D: Debug>(&mut self, value: D) -> Result<()> {
        self.item(DebugToDisplay(value))
    }
}

struct DebugToDisplay<T>(T);

impl<T: Debug> Display for DebugToDisplay<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.0))
    }
}
