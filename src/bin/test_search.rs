use aurdash::aur::search_repos;

#[tokio::main]
async fn main() {
    let pkgs = search_repos("firefox").await.unwrap();
    println!("Got {} repo packages", pkgs.len());
    for p in pkgs.into_iter().take(3) {
        println!("{:?}", p);
    }
}
