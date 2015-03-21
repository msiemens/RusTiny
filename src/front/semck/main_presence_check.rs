//! Make sure a `main` function is present

use std::borrow::ToOwned;
use front::ast::*;
use front::ast::visit::*;


struct MainPresenceCheck {
    main_present: bool
}

impl MainPresenceCheck {
    fn new() -> MainPresenceCheck {
        MainPresenceCheck {
            main_present: false
        }
    }
}

impl<'v> Visitor<'v> for MainPresenceCheck {
    fn visit_symbol(&mut self, s: &'v Node<Symbol>) {
        if let Symbol::Function { ref name, .. } = **s {
            if &***name == "main" {
                self.main_present = true;
            }
        }
    }
}

pub fn run(program: &Program) {
    let mut visitor = MainPresenceCheck::new();
    walk_program(&mut visitor, program);

    if !visitor.main_present {
        fatal!("main function not found");
    }
}