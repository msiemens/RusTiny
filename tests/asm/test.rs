// RusTiny IR:
// 
// fn func() {                                        
// entry-block:                                       
//     %ret_slot = alloca                             
//     store 10 %ret_slot                             
//     jmp return                                     
// return:                                            
//     %0 = load %ret_slot                            
//     ret %0                                         
// }                                                  
//                                                    
// fn main() {                                        
// entry-block:                                       
//     %1 = cmp eq 1 2                                
//     br %1 lazy-next1 lazy-rhs1                     
// lazy-rhs1:                                         
//     jmp lazy-next1                                 
// lazy-next1:                                        
//     %0 = phi [ %1, entry-block ] [ 0, lazy-rhs1 ]  
//     br %0 conseq1 next1                            
// conseq1:                                           
//     %2 = call func                                 
//     jmp next1                                      
// next1:                                             
//     ret void                                       
// }                                                  


fn func() -> int {
    10
}

fn main() {
    let i: int = 2;
    if 1 == i || false {
        func()
    }
}