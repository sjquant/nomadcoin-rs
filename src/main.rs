use nomadcoin::BlockChain;

fn main() {
    let mut chain = BlockChain::new();
    chain.add_block("Hello, World");
    chain.add_block("Another Block");
    chain.add_block("Third Block");
    chain.print_blocks();
}