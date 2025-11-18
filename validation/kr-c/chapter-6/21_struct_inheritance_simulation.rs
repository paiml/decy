/* K&R C Chapter 6: Simulating Inheritance with Structs
 * Base struct as first member for "inheritance"
 * Transpiled to safe Rust (using traits)
 */

// Base "class"
#[derive(Debug, Clone)]
struct Entity {
    id: i32,
    name: String,
}

impl Entity {
    fn new(id: i32, name: &str) -> Self {
        Entity {
            id,
            name: name.to_string(),
        }
    }

    fn print(&self) {
        println!("Entity #{}: {}", self.id, self.name);
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }
}

// Derived "classes"
#[derive(Debug, Clone)]
struct Player {
    entity: Entity,
    health: f32,
    damage: f32,
}

impl Player {
    fn new(id: i32, name: &str, health: f32, damage: f32) -> Self {
        Player {
            entity: Entity::new(id, name),
            health,
            damage,
        }
    }

    fn print(&self) {
        println!(
            "Player #{}: {} (HP:{:.1}, DMG:{:.1})",
            self.entity.id, self.entity.name, self.health, self.damage
        );
    }
}

#[derive(Debug, Clone)]
struct Vehicle {
    entity: Entity,
    speed: f32,
    cargo_capacity: i32,
}

impl Vehicle {
    fn new(id: i32, name: &str, speed: f32, cargo_capacity: i32) -> Self {
        Vehicle {
            entity: Entity::new(id, name),
            speed,
            cargo_capacity,
        }
    }

    fn print(&self) {
        println!(
            "Vehicle #{}: {} (Speed:{:.1}, Cargo:{})",
            self.entity.id, self.entity.name, self.speed, self.cargo_capacity
        );
    }
}

#[derive(Debug, Clone)]
struct Enemy {
    entity: Entity,
    level: i32,
    xp: i32,
}

impl Enemy {
    fn new(id: i32, name: &str, level: i32, xp: i32) -> Self {
        Enemy {
            entity: Entity::new(id, name),
            level,
            xp,
        }
    }

    fn print(&self) {
        println!(
            "Enemy #{}: {} (Level:{}, XP:{})",
            self.entity.id, self.entity.name, self.level, self.xp
        );
    }
}

// Trait for polymorphism
trait EntityTrait {
    fn entity(&self) -> &Entity;
    fn entity_mut(&mut self) -> &mut Entity;

    fn print_entity(&self) {
        self.entity().print();
    }

    fn set_name(&mut self, name: &str) {
        self.entity_mut().set_name(name);
    }
}

impl EntityTrait for Player {
    fn entity(&self) -> &Entity {
        &self.entity
    }
    fn entity_mut(&mut self) -> &mut Entity {
        &mut self.entity
    }
}

impl EntityTrait for Vehicle {
    fn entity(&self) -> &Entity {
        &self.entity
    }
    fn entity_mut(&mut self) -> &mut Entity {
        &mut self.entity
    }
}

impl EntityTrait for Enemy {
    fn entity(&self) -> &Entity {
        &self.entity
    }
    fn entity_mut(&mut self) -> &mut Entity {
        &mut self.entity
    }
}

fn main() {
    println!("=== Inheritance Simulation ===\n");

    // Create entities
    let mut player = Player::new(1, "Hero", 100.0, 25.0);
    let vehicle = Vehicle::new(2, "Truck", 80.0, 1000);
    let enemy = Enemy::new(3, "Goblin", 5, 50);

    // Use base class methods via trait
    println!("Using generic Entity trait:");
    player.print_entity();
    vehicle.entity().print();
    enemy.entity().print();

    println!("\nUsing type-specific functions:");
    player.print();
    vehicle.print();
    enemy.print();

    // Modify through base
    println!("\nModifying through base:");
    player.set_name("SuperHero");
    player.print();

    // Polymorphism via trait objects
    let entities: Vec<&dyn EntityTrait> = vec![&player, &vehicle, &enemy];

    println!("\nPolymorphism via trait objects:");
    for entity in entities {
        entity.print_entity();
    }
}

// Alternative: enum for closed set of types
#[allow(dead_code)]
fn demonstrate_enum_polymorphism() {
    #[derive(Debug)]
    enum GameEntity {
        Player { id: i32, name: String, health: f32 },
        Vehicle { id: i32, name: String, speed: f32 },
        Enemy { id: i32, name: String, level: i32 },
    }

    impl GameEntity {
        fn id(&self) -> i32 {
            match self {
                GameEntity::Player { id, .. } => *id,
                GameEntity::Vehicle { id, .. } => *id,
                GameEntity::Enemy { id, .. } => *id,
            }
        }

        fn name(&self) -> &str {
            match self {
                GameEntity::Player { name, .. } => name,
                GameEntity::Vehicle { name, .. } => name,
                GameEntity::Enemy { name, .. } => name,
            }
        }

        fn print(&self) {
            println!("Entity #{}: {}", self.id(), self.name());
        }
    }

    let entities = vec![
        GameEntity::Player {
            id: 1,
            name: "Hero".to_string(),
            health: 100.0,
        },
        GameEntity::Vehicle {
            id: 2,
            name: "Truck".to_string(),
            speed: 80.0,
        },
    ];

    for entity in &entities {
        entity.print();
    }
}

// Key differences from C:
// 1. Traits for polymorphism (not base pointers)
// 2. enum for closed type hierarchies
// 3. trait objects (dyn Trait) for runtime polymorphism
// 4. No pointer casting needed
// 5. Type safety enforced at compile time
// 6. No memory layout assumptions
// 7. Composition preferred over inheritance
// 8. Pattern matching for type checking
