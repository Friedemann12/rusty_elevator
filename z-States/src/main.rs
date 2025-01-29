use rand::Rng;
use std::collections::VecDeque;
use ggez::{Context, GameResult, graphics::{self, Color, Text}, event};
use ggez::glam::Vec2;

// Definition of cabin states 
#[derive(Debug, Clone, PartialEq)]
enum CabinState {
    Standing(i32),
    Moving(i32, i32),
    Holding(i32),
}

// Definition of door states
#[derive(Debug, Clone, PartialEq)]
enum DoorState {
    Closed,
    Opening,
    Open,
    Closing,
}

// Definition of passenger states
#[derive(Debug, Clone, PartialEq)]
enum PassengerState {
    Idle(i32),
    Entering,
    ChoosingFloor,
    InCabin,
    Exiting,
}

// Definition of movement directions
#[derive(Debug, Clone, PartialEq)]
enum Direction_ {
    UP,
    DOWN,
}

// Structure for passenger
#[derive(Debug, Clone)]
struct Passenger {
    id: usize,
    state: PassengerState,
    direction: Direction_,
    destination: i32,
}

// Implementation of passenger functionality
impl Passenger {
    fn new(id: usize, floor: i32, destination_floor: i32) -> Self {
        Passenger {
            id,
            state: PassengerState::Idle(floor),
            destination: destination_floor,

            // Direction of passenger is based on start and destination floors
            direction: if floor < destination_floor {
                Direction_::UP
            } else {
                Direction_::DOWN
            },
        }
    }

    //fn destination(&self) -> Option<i32> {
    //    if let PassengerState::ChoosingFloor(dest) = self.state {
    //        Some(dest)
    //    } else {
    //        None
    //    }
    //}
}

// Structure for an elevator
#[derive(Debug)]
struct Elevator {
    cabin_state: CabinState,
    door_state: DoorState,
    passengers: Vec<Passenger>,
    destinations: VecDeque<i32>,
    max_capacity: usize,
    current_direction: Option<Direction_>,
    door_timer: u32,
}

// Implementation of elevator functionality
impl Elevator {
    fn new(starting_floor: i32) -> Self {
        Elevator {
            cabin_state: CabinState::Standing(starting_floor),
            door_state: DoorState::Closed,
            passengers: vec![],
            destinations: VecDeque::new(),
            max_capacity: 2,  // Requirement S5
            current_direction: None,
            door_timer: 0,
        }
    }

    // Checks if elevator is full
    fn is_full(&self) -> bool {
        self.passengers.len() >= self.max_capacity
    }

    // Sorts destination floors in optimal order
    fn sort_destinations(&mut self) {
        if let Some(current_floor) = match self.cabin_state {
            CabinState::Standing(f) | CabinState::Holding(f) => Some(f),
            CabinState::Moving(from, _) => Some(from),
        } {
            // Convert destinations to Vec for sorting
            let mut dest_vec: Vec<i32> = self.destinations.drain(..).collect();
            
            // Sort based on current direction and position
            let going_up = dest_vec.iter().any(|&d| d > current_floor);
            
            if going_up {
                // Sort in ascending order for floors above current position
                dest_vec.sort_by(|a, b| {
                    if a >= &current_floor && b >= &current_floor {
                        a.cmp(b)
                    } else if a < &current_floor && b < &current_floor {
                        b.cmp(a)
                    } else {
                        if a >= &current_floor {
                            std::cmp::Ordering::Less
                        } else {
                            std::cmp::Ordering::Greater
                        }
                    }
                });
            } else {
                // Sort in descending order for floors below current position
                dest_vec.sort_by(|a, b| {
                    if a <= &current_floor && b <= &current_floor {
                        b.cmp(a)
                    } else if a > &current_floor && b > &current_floor {
                        a.cmp(b)
                    } else {
                        if a <= &current_floor {
                            std::cmp::Ordering::Less
                        } else {
                            std::cmp::Ordering::Greater
                        }
                    }
                });
            }

            // Put back into destinations queue
            self.destinations = VecDeque::from(dest_vec);
        }
    }

    // Adds a new destination floor
    fn add_destination(&mut self, floor: i32) {
        if !self.destinations.contains(&floor) {
            self.destinations.push_back(floor);
            self.sort_destinations();  // Sort destinations after adding new one
        }
    }

    // SIMULATION
    // Executes a simulation step
    fn step(&mut self) {
        match self.cabin_state.clone() {
            // When elevator is standing
            CabinState::Standing(floor) => {
                if self.door_state == DoorState::Closed {
                    if let Some(&next_floor) = self.destinations.front() {
                        // Check if there's a valid reason to go to this floor
                        let has_waiting_passenger = self.destinations.contains(&next_floor);
                        let has_passenger_going_there = self.passengers.iter().any(|p| p.destination == next_floor);
                        
                        if has_waiting_passenger || has_passenger_going_there {
                            if next_floor == floor {
                                self.cabin_state = CabinState::Holding(floor);
                                self.door_state = DoorState::Opening;
                            } else {
                                self.cabin_state = CabinState::Moving(floor, next_floor);
                            }
                        } else {
                            // If no valid reason to go to this floor, remove it from destinations
                            self.destinations.pop_front();
                        }
                    }
                }
            }
            // When elevator is moving
            CabinState::Moving(current, target) => {
                let new_floor = if current < target {
                    current + 1
                } else {
                    current - 1
                };
                if new_floor == target {
                    self.cabin_state = CabinState::Holding(target);
                    // Double check if we still need to stop here
                    let has_waiting_passenger = self.destinations.contains(&target);
                    let has_passenger_going_there = self.passengers.iter().any(|p| p.destination == target);
                    if !has_waiting_passenger && !has_passenger_going_there {
                        self.destinations.pop_front();
                        self.cabin_state = CabinState::Standing(target);
                    }
                } else {
                    self.cabin_state = CabinState::Standing(new_floor);
                }
            }
            // When elevator is holding for passenger exchange
            CabinState::Holding(floor) => match self.door_state {
                DoorState::Opening => {
                    self.door_state = DoorState::Open;
                }
                DoorState::Open => {
                    self.handle_passenger_exchange(floor);
                    self.door_state = DoorState::Closing;
                }
                DoorState::Closing => {
                    self.door_state = DoorState::Closed;
                    self.cabin_state = CabinState::Standing(floor);
                    self.destinations.pop_front(); // Remove reached destination
                }
                DoorState::Closed => {
                    self.door_state = DoorState::Opening;
                }
            },
        }
        
        // Fix the passenger state checking
        for passenger in &mut self.passengers {
            if let PassengerState::Idle(floor) = passenger.state {  // Fixed pattern matching
                match self.cabin_state.clone() {
                    CabinState::Standing(current_floor) => {
                        if floor == current_floor {
                            self.destinations.push_back(passenger.destination);
                        }
                    },
                    CabinState::Moving(start, end) => {
                        let is_going_up = end > start;
                        let passenger_going_up = passenger.destination > floor;
                        
                        // Add destination if elevator is moving in same direction as passenger wants
                        if (is_going_up && passenger_going_up) || (!is_going_up && !passenger_going_up) {
                            if !self.destinations.contains(&floor) {
                                self.destinations.push_back(floor);
                            }
                        }
                    },
                    CabinState::Holding(_) => {} // Do nothing if holding
                }
            }
        }
    }

    fn handle_passenger_exchange(&mut self, current_floor: i32) -> Vec<usize> {
        // Handle exiting passengers
        let mut exited_passengers = Vec::new();
        self.passengers.retain(|passenger| {
            if passenger.state == PassengerState::InCabin && passenger.destination == current_floor {
                println!("Passenger {} exiting at floor {}", passenger.id, current_floor);
                exited_passengers.push(passenger.id);
                false
            } else {
                true
            }
        });

        // Debug print
        println!("Elevator at floor {} has {} passengers", current_floor, self.passengers.len());

        self.door_timer += 1;
        if self.door_timer > 5 && !self.is_full() {
            self.door_state = DoorState::Closing;
            self.door_timer = 0;
        }
        
        exited_passengers
    }
}

// Structure for the control system
struct ControlSystem {
    passengers: Vec<Passenger>,
    elevators: Vec<Elevator>,
    passenger_counter: usize,
}


// Implementation of control system functionality
impl ControlSystem {
    fn new(num_elevators: usize) -> Self {
        let elevators = (0..num_elevators)
            .map(|_| Elevator::new(0))
            .collect();

        ControlSystem {
            passengers: Vec::new(),
            elevators,
            passenger_counter: 0,
        }
    }

    // Adds a random passenger
    fn add_random_passenger(&mut self) {
        let floor = rand::thread_rng().gen_range(0..4);
        let mut destination_floor = rand::thread_rng().gen_range(0..4);
        while floor == destination_floor {
            destination_floor = rand::thread_rng().gen_range(0..4);
        }
        self.passengers.push(Passenger::new(
            self.passenger_counter,
            floor,
            destination_floor,
        ));
        self.passenger_counter += 1;
    }

    // Executes a simulation step
    fn step(&mut self) {
        // Add new random passenger with lower probability
        if rand::thread_rng().gen_bool(0.2) {
            self.add_random_passenger();
        }

        // First assign passengers to elevators
        self.assign_passengers_to_elevators();

        // Then update each elevator
        for elevator in &mut self.elevators {
            elevator.step();

            if let (CabinState::Holding(floor), DoorState::Open) = (&elevator.cabin_state, &elevator.door_state) {
                if !elevator.is_full() {
                    let current_floor = *floor;
                    let mut passengers_to_remove = Vec::new();
                    let mut passengers_to_add = Vec::new();

                    // First, identify all passengers that should enter
                    for (index, passenger) in self.passengers.iter().enumerate() {
                        if let PassengerState::Idle(p_floor) = passenger.state {
                            if p_floor == current_floor && passengers_to_add.len() + elevator.passengers.len() < elevator.max_capacity {
                                passengers_to_remove.push(index);
                                let mut new_passenger = passenger.clone();
                                new_passenger.state = PassengerState::InCabin;
                                passengers_to_add.push(new_passenger);
                            }
                        }
                    }

                    // Remove passengers from waiting list in reverse order
                    for &index in passengers_to_remove.iter().rev() {
                        let passenger = &self.passengers[index];
                        println!("Passenger {} entering elevator at floor {}", passenger.id, current_floor);
                        self.passengers.remove(index);
                    }

                    // Add passengers to elevator and their destinations
                    for passenger in passengers_to_add {
                        elevator.add_destination(passenger.destination);  // Add destination before adding passenger
                        elevator.passengers.push(passenger);
                    }
                }
            }
        }
    }

    // Assigns waiting passengers to elevators
    fn assign_passengers_to_elevators(&mut self) {
        for passenger in &self.passengers {
            if let PassengerState::Idle(floor) = passenger.state {
                // Find best elevator for this passenger
                let best_elevator = self.elevators
                    .iter_mut()
                    .filter(|e| !e.is_full())
                    .min_by_key(|e| {
                        match e.cabin_state {
                            CabinState::Standing(e_floor) | CabinState::Holding(e_floor) => {
                                (e_floor - floor).abs()
                            },
                            CabinState::Moving(from, to) => {
                                // Check if passenger is "on the way"
                                let elevator_going_up = to > from;
                                let passenger_going_up = passenger.destination > floor;
                                
                                if elevator_going_up == passenger_going_up {  // Same direction
                                    if elevator_going_up {
                                        // Going up: passenger should be between current position and destination
                                        if floor >= from && floor <= to {
                                            0  // Perfect match!
                                        } else {
                                            i32::MAX  // Wrong direction
                                        }
                                    } else {
                                        // Going down: passenger should be between destination and current position
                                        if floor <= from && floor >= to {
                                            0  // Perfect match!
                                        } else {
                                            i32::MAX  // Wrong direction
                                        }
                                    }
                                } else {
                                    i32::MAX  // Wrong direction
                                }
                            }
                        }
                    });

                if let Some(elevator) = best_elevator {
                    elevator.add_destination(floor);  // Add pickup floor as destination
                }
            }
        }
    }
}

struct GameState {
    control_system: ControlSystem,
    step_timer: f32,
}

impl GameState {
    fn new() -> Self {
        GameState {
            control_system: ControlSystem::new(3),
            step_timer: 0.0,
        }
    }
}

impl event::EventHandler<ggez::GameError> for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.step_timer += ctx.time.delta().as_secs_f32();
        if self.step_timer >= 1.5 {  // Slightly faster than 2.0 but still slow enough to see
            self.control_system.step();
            self.step_timer = 0.0;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::WHITE);
        let draw_param = graphics::DrawParam::default();
        
        // Draw building
        let floor_height = 100.0;
        let building_left = 100.0;
        let building_width = 400.0;

        // Draw floors
        for floor in 0..4 {
            let y = 500.0 - (floor as f32 * floor_height);
            canvas.draw(
                &graphics::Mesh::new_line(
                    ctx,
                    &[Vec2::new(building_left, y), Vec2::new(building_left + building_width, y)],
                    2.0,
                    Color::BLACK,
                )?,
                draw_param,
            );
        }

        // Draw floor numbers
        for floor in 0..4 {
            let y = 500.0 - (floor as f32 * floor_height);
            let floor_text = Text::new(format!("Floor {}", floor));
            canvas.draw(
                &floor_text,
                graphics::DrawParam::default()
                    .dest(Vec2::new(20.0, y - 10.0))
                    .color(Color::BLACK),
            );
        }

        // Draw elevators with improved passenger visualization
        let elevator_width = 60.0;
        let elevator_spacing = (building_width - 3.0 * elevator_width) / 4.0;

        for (i, elevator) in self.control_system.elevators.iter().enumerate() {
            let elevator_x = building_left + elevator_spacing + (i as f32 * (elevator_width + elevator_spacing));
            
            // Simplified elevator position calculation - no interpolation
            let elevator_y = match &elevator.cabin_state {
                CabinState::Standing(floor) | CabinState::Holding(floor) => 
                    500.0 - (*floor as f32 * floor_height),
                CabinState::Moving(from, to) => 
                    // Just show elevator at the 'from' floor - no animation
                    500.0 - (*from as f32 * floor_height),
            };

            // Add movement direction indicator
            let direction_text = match &elevator.cabin_state {
                CabinState::Moving(from, to) if from < to => "▲",
                CabinState::Moving(from, to) if from > to => "▼",
                _ => "",
            };
            
            canvas.draw(
                &Text::new(direction_text),
                graphics::DrawParam::default()
                    .dest(Vec2::new(elevator_x - 5.0, elevator_y - 60.0))
                    .color(Color::BLACK),
            );

            // Draw elevator state (debug info)
            let state_text = match &elevator.cabin_state {
                CabinState::Standing(floor) => format!("Standing {}", floor),
                CabinState::Moving(from, to) => format!("Moving {}→{}", from, to),
                CabinState::Holding(floor) => format!("Holding {}", floor),
            };
            
            canvas.draw(
                &Text::new(state_text),
                graphics::DrawParam::default()
                    .dest(Vec2::new(elevator_x - 30.0, elevator_y - 55.0))
                    .color(Color::BLACK),
            );

            // Draw destinations queue
            let dest_text = format!("Dest: {:?}", elevator.destinations);
            canvas.draw(
                &Text::new(dest_text),
                graphics::DrawParam::default()
                    .dest(Vec2::new(elevator_x - 30.0, 80.0))
                    .color(Color::BLACK),
            );

            // Draw elevator shaft
            canvas.draw(
                &graphics::Mesh::new_line(
                    ctx,
                    &[
                        Vec2::new(elevator_x, 500.0),
                        Vec2::new(elevator_x, 100.0),
                    ],
                    1.0,
                    Color::new(0.5, 0.5, 0.5, 1.0),  // RGB values for gray (0.5, 0.5, 0.5) with alpha 1.0
                )?,
                draw_param,
            );

            // Draw elevator cabin
            let elevator_color = if elevator.door_state == DoorState::Open {
                Color::GREEN
            } else {
                Color::BLUE
            };

            canvas.draw(
                &graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    graphics::Rect::new(elevator_x - 20.0, elevator_y - 40.0, 40.0, 80.0),
                    elevator_color,
                )?,
                draw_param,
            );

            // Draw elevator number
            let elevator_text = Text::new(format!("E{}", i));
            canvas.draw(
                &elevator_text,
                graphics::DrawParam::default()
                    .dest(Vec2::new(elevator_x - 15.0, elevator_y - 35.0))
                    .color(Color::WHITE),
            );

            // Draw passengers in elevator with destination indicators
            for (p_idx, passenger) in elevator.passengers.iter().enumerate().take(elevator.max_capacity) {
                let passenger_x = elevator_x - 10.0 + (p_idx as f32 * 20.0);
                
                // Draw passenger dot
                canvas.draw(
                    &graphics::Mesh::new_circle(
                        ctx,
                        graphics::DrawMode::fill(),
                        Vec2::new(passenger_x, elevator_y),
                        5.0,
                        0.1,
                        Color::BLACK,
                    )?,
                    draw_param,
                );

                // Draw destination indicator
                let dest_text = Text::new(format!("→{}", passenger.destination));
                canvas.draw(
                    &dest_text,
                    graphics::DrawParam::default()
                        .dest(Vec2::new(passenger_x - 5.0, elevator_y + 10.0))
                        .color(Color::BLACK),
                );
            }
        }

        // Draw waiting passengers with destination indicators
        for passenger in &self.control_system.passengers {
            if let PassengerState::Idle(floor) = passenger.state {
                let y = 500.0 - (floor as f32 * floor_height);
                
                // Draw passenger dot
                canvas.draw(
                    &graphics::Mesh::new_circle(
                        ctx,
                        graphics::DrawMode::fill(),
                        Vec2::new(50.0, y),
                        5.0,
                        0.1,
                        Color::RED,
                    )?,
                    draw_param,
                );

                // Draw destination indicator
                let dest_text = Text::new(format!("→{}", passenger.destination));
                canvas.draw(
                    &dest_text,
                    graphics::DrawParam::default()
                        .dest(Vec2::new(60.0, y - 5.0))
                        .color(Color::RED),
                );
            }
        }

        canvas.finish(ctx)?;
        Ok(())
    }
}

fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("Elevator Simulation", "Your Name")
        .window_setup(ggez::conf::WindowSetup::default().title("Elevator Simulation"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(600.0, 600.0));
    
    let (ctx, event_loop) = cb.build()?;
    let state = GameState::new();
    event::run(ctx, event_loop, state)
}
