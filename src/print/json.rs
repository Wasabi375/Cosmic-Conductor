#![allow(unused, unreachable_code)] // TODO temp

use super::{Print, PrintList};

struct Printer {}

impl Print for Printer {
    fn print_field<D: std::fmt::Display>(&self, name: &str, value: D) {
        todo!()
    }

    fn sub_struct(&self, name: &str) -> impl Print {
        todo!();
        Printer {}
    }

    fn sub_list(&self, name: &str) -> impl super::PrintList {
        todo!();
        ListPrinter {}
    }

    fn print_list<D: std::fmt::Display>(&self, name: &str, value: impl Iterator<Item = D>) {
        todo!()
    }
}

struct ListPrinter {}

impl PrintList for ListPrinter {
    fn print_item<D: std::fmt::Display>(&self, value: D) {
        todo!()
    }

    fn sub_struct(&self, name: &str) -> impl Print {
        todo!();
        Printer {}
    }

    fn sub_list(&self) -> impl PrintList {
        todo!();
        ListPrinter {}
    }
}
