// TODO: Optimizations
// TODO: Insert intrinsics implementations
// TODO: Replace intrinsics usage with appropriate calls


pub use middle::liveness::LivenessAnalysis;


pub mod ir;
mod liveness;


pub fn calculate_liveness(code: &ir::Program) -> LivenessAnalysis {
    liveness::LivenessAnalyzer::run(code)
}