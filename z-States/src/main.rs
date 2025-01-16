use rand::Rng;
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
enum CabinState {
    Standing(i32),
    Moving(i32, i32),
    Holding(i32),
}

#[derive(Debug, Clone, PartialEq)]
enum DoorState {
    Closed,
    Opening,
    Open,
    Closing,
}

#[derive(Debug, Clone, PartialEq)]
enum PassengerState {
    Idle(i32),
    Entering,
    ChoosingFloor(i32),
    InCabin,
    Exiting(i32),
}

#[derive(Debug)]
struct Passenger {
    id: usize,
    state: PassengerState,
}

impl Passenger {
    fn new(id: usize, floor: i32) -> Self {
        Passenger {
            id,
            state: PassengerState::Idle(floor),
        }
    }

    fn destination(&self) -> Option<i32> {
        if let PassengerState::ChoosingFloor(dest) = self.state {
            Some(dest)
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct Elevator {
    cabin_state: CabinState,
    door_state: DoorState,
    passengers: Vec<Passenger>,
    destinations: VecDeque<i32>,
}

impl Elevator {
    fn new(starting_floor: i32) -> Self {
        Elevator {
            cabin_state: CabinState::Standing(starting_floor),
            door_state: DoorState::Closed,
            passengers: vec![],
            destinations: VecDeque::new(),
        }
    }

    fn add_destination(&mut self, floor: i32) {
        if !self.destinations.contains(&floor) {
            self.destinations.push_back(floor);
        }
    }

    fn step(&mut self) {
        match self.cabin_state.clone() {
            CabinState::Standing(floor) => {
                if self.door_state == DoorState::Closed {
                    if let Some(&next_floor) = self.destinations.front() {
                        if next_floor == floor {
                            self.cabin_state = CabinState::Holding(floor);
                            self.door_state = DoorState::Opening;
                        } else {
                            self.cabin_state = CabinState::Moving(floor, next_floor);
                        }
                    }
                }
            }
            CabinState::Moving(current, target) => {
                let new_floor = if current < target { current + 1 } else { current - 1 };
                if new_floor == target {
                    self.cabin_state = CabinState::Holding(target);
                } else {
                    self.cabin_state = CabinState::Standing(new_floor);
                }
            }
            CabinState::Holding(floor) => match self.door_state {
                DoorState::Opening => {
                    self.door_state = DoorState::Open;
                }
                DoorState::Open => {
                    self.handle_passenger_exchange(floor); //TODO - Funktioniert nicht
                    self.door_state = DoorState::Closing;
                }
                DoorState::Closing => {
                    self.door_state = DoorState::Closed;
                    self.cabin_state = CabinState::Standing(floor);
                    self.destinations.pop_front(); // Remove reached destination
                }
                DoorState::Closed => {
                    self.door_state = DoorState::Opening;
                },
            },
        }
    }

    fn handle_passenger_exchange(&mut self, current_floor: i32) {
        // Passengers exiting
        self.passengers.retain(|passenger| {
            if let PassengerState::InCabin = passenger.state {
                if let Some(target) = passenger.destination() {
                    if target == current_floor {
                        println!("Passenger {} exiting at floor {}", passenger.id, current_floor);
                        return false; // Remove passenger from cabin
                    }
                }
            }
            true
        });

        // Passengers entering and collecting new destinations
        let mut new_destinations = Vec::new();
        for passenger in self.passengers.iter_mut() {
            if let PassengerState::Idle(floor) = passenger.state {
                if floor == current_floor {
                    passenger.state = PassengerState::Entering;
                    println!("Passenger {} entering at floor {}", passenger.id, current_floor);
                    passenger.state = PassengerState::ChoosingFloor(rand::thread_rng().gen_range(0..4));
                }
            }

            // Add new destinations for entered passengers
            if let PassengerState::ChoosingFloor(dest) = passenger.state {
                passenger.state = PassengerState::InCabin;
                new_destinations.push(dest);
            }
        }

        // After looping over passengers, add new destinations to the elevator
        for dest in new_destinations {
            self.add_destination(dest);
        }
    }

}

struct ControlSystem {
    passengers: Vec<Passenger>,
    elevators: Vec<Elevator>,
    passenger_counter: usize,
}

impl ControlSystem {
    fn new(num_elevators: usize) -> Self {
        let elevators = (0..num_elevators)
            .map(|_| Elevator::new(0)) // All elevators start at floor 0
            .collect();

        ControlSystem {
            passengers: Vec::new(),
            elevators,
            passenger_counter: 0,
        }
    }

    fn add_random_passenger(&mut self) {
        let floor = rand::thread_rng().gen_range(0..4);
        self.passengers
            .push(Passenger::new(self.passenger_counter, floor));
        self.passenger_counter += 1;
    }

    fn assign_passengers_to_elevators(&mut self) {
        for passenger in &self.passengers {
            if let PassengerState::Idle(starting_floor) = passenger.state {
                let nearest_elevator = self
                    .elevators
                    .iter_mut()
                    .min_by_key(|e| match e.cabin_state {
                        CabinState::Standing(floor) => (floor - starting_floor).abs(),
                        _ => i32::MAX,
                    });
                if let Some(elevator) = nearest_elevator {
                    elevator.add_destination(starting_floor);
                }
            }
        }
    }

    fn step(&mut self) {
        // Update each elevator
        for elevator in &mut self.elevators {
            elevator.step();
        }

        // Assign passengers to elevators
        self.assign_passengers_to_elevators();

        // Add new random passenger occasionally
        if rand::thread_rng().gen_bool(0.3) {
            self.add_random_passenger();
        }
    }
}

fn main() {
    let mut control_system = ControlSystem::new(1);

    for step in 0..20 {
        println!("Step {}", step);
        control_system.step();

        for (i, elevator) in control_system.elevators.iter().enumerate() {
            println!(
                "Elevator {} - Cabin: {:?}, Doors: {:?}, Passengers: {:?}, Destinations: {:?}",
                i, elevator.cabin_state, elevator.door_state, elevator.passengers, elevator.destinations
            );
        }
        println!("Passengers: {:?}", control_system.passengers);
    }
}
