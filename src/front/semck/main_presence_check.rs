//! Make sure a `main` function is present

use front::ast::visit::*;
use front::ast::*;

struct MainPresenceCheck {
    main_present: bool,
}

impl MainPresenceCheck {
    fn new() -> MainPresenceCheck {
        MainPresenceCheck {
            main_present: false,
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

pub fn run(program: &[Node<Symbol>]) {
    let mut visitor = MainPresenceCheck::new();
    walk_program(&mut visitor, program);

    if !visitor.main_present {
        fatal!("main function not found");
    }
}
