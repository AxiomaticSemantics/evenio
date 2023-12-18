use evenio::prelude::*;

#[derive(Event)]
struct A;

#[derive(Event)]
struct B;

#[derive(Event)]
struct C;

pub fn main() {
    let mut world = World::new();

    world.add_system(get_a_send_b);
    world.add_system(get_b_send_c);
    world.add_system(get_c);

    println!("sending A!");
    world.send(A);
}

fn get_a_send_b(_: Receiver<A>, mut sender: Sender<B>) {
    println!("got A, sending B twice!");
    sender.send(B);
    sender.send(B);
}

fn get_b_send_c(_: Receiver<B>, mut sender: Sender<C>) {
    println!("got B, sending C twice!");
    sender.send(C);
    sender.send(C);
}

fn get_c(_: Receiver<C>) {
    println!("got C!");
}
