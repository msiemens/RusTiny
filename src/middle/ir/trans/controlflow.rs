//! Translate of control flow statements/expressions

// FIXME: Docs

use ::Ident;
use front::ast;
use middle::ir;
use middle::ir::trans::{Dest, Translator};

impl Translator {
    /// Translate a return statement
    pub fn trans_return(&mut self,
                        val: &ast::Expression,
                        block: &mut ir::Block) {
        // Store the return value in the return slot and jump to the return block
        let val = self.trans_expr_to_value(val, block);
        let return_slot = self.fcx().return_slot.unwrap();

        block.store_reg(val, return_slot);
        block.jump(ir::Label(Ident::new("return")));
    }

    /// Translate an if expression
    pub fn trans_if(&mut self,
                    cond: &ast::Expression,
                    conseq: &ast::Node<ast::Block>,
                    altern: Option<&ast::Node<ast::Block>>,
                    block: &mut ir::Block,
                    dest: Dest) {
        let cond_ir = self.trans_expr_to_value(cond, block);

        match altern {
            Some(altern) => {
                let label_conseq = self.next_free_label(Ident::new("conseq"));
                let label_altern = self.next_free_label(Ident::new("altern"));
                let label_next = self.next_free_label(Ident::new("next"));

                block.branch(cond_ir, label_conseq, label_altern);

                // The 'then' block
                self.commit_block_and_continue(block, label_conseq);
                self.trans_block(conseq, block, dest);
                // FIXME: Better solution?
                if !block.finalized() {
                    block.jump(label_next);  // Skip the 'else' part
                }

                // The 'else' block
                self.commit_block_and_continue(block, label_altern);
                self.trans_block(altern, block, dest);
                if !block.finalized() {
                    block.jump(label_next);
                }

                self.commit_block_and_continue(block, label_next);
            },
            None => {
                let label_conseq = self.next_free_label(Ident::new("conseq"));
                let label_next = self.next_free_label(Ident::new("next"));

                block.branch(cond_ir, label_conseq, label_next);

                // The 'then' block
                self.commit_block_and_continue(block, label_conseq);
                self.trans_block(conseq, block, dest);
                if !block.finalized() {
                    block.jump(label_next);
                }

                self.commit_block_and_continue(block, label_next);
            }
        }
    }

    /// Translate a while expression
    pub fn trans_while(&mut self,
                       cond: &ast::Expression,
                       body: &ast::Node<ast::Block>,
                       block: &mut ir::Block) {
        let label_cond = self.next_free_label(Ident::new("while_cond"));
        let label_body = self.next_free_label(Ident::new("while_body"));
        let label_next = self.next_free_label(Ident::new("while_exit"));

        block.jump(label_cond);

        with_reset!(self.fcx().loop_exit, Some(label_next), {
            // Condition block
            self.commit_block_and_continue(block, label_cond);
            let cond = self.trans_expr_to_value(cond, block);
            block.branch(cond, label_body, label_next);

            // Body block
            self.commit_block_and_continue(block, label_body);
            self.trans_block(body, block, Dest::Ignore);
            if !block.finalized() {
                // After executing the body, re-check the condition
                block.jump(label_cond);
            }

            // Exit block
            self.commit_block_and_continue(block, label_next);
        });
    }

    /// Translate a break expression
    pub fn trans_break(&mut self,
                       block: &mut ir::Block) {
        block.jump(self.fcx().loop_exit.unwrap());
    }
}