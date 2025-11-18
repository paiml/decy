/* K&R C Chapter 6: Simulating Inheritance with Structs
 * Base struct as first member for "inheritance"
 */

#include <stdio.h>
#include <string.h>

/* Base "class" */
typedef struct {
    int id;
    char name[50];
} Entity;

/* Derived "classes" - first member is base */
typedef struct {
    Entity base;  /* Must be first member */
    float health;
    float damage;
} Player;

typedef struct {
    Entity base;  /* Must be first member */
    float speed;
    int cargo_capacity;
} Vehicle;

typedef struct {
    Entity base;  /* Must be first member */
    int level;
    int xp;
} Enemy;

/* Generic functions that work on Entity */
void entity_print(Entity *e) {
    printf("Entity #%d: %s\n", e->id, e->name);
}

void entity_set_name(Entity *e, const char *name) {
    strncpy(e->name, name, sizeof(e->name) - 1);
    e->name[sizeof(e->name) - 1] = '\0';
}

/* Type-specific functions */
void player_print(Player *p) {
    printf("Player #%d: %s (HP:%.1f, DMG:%.1f)\n",
           p->base.id, p->base.name, p->health, p->damage);
}

void vehicle_print(Vehicle *v) {
    printf("Vehicle #%d: %s (Speed:%.1f, Cargo:%d)\n",
           v->base.id, v->base.name, v->speed, v->cargo_capacity);
}

void enemy_print(Enemy *e) {
    printf("Enemy #%d: %s (Level:%d, XP:%d)\n",
           e->base.id, e->base.name, e->level, e->xp);
}

int main() {
    printf("=== Inheritance Simulation ===\n\n");

    /* Create entities */
    Player player = {
        .base = {1, "Hero"},
        .health = 100.0,
        .damage = 25.0
    };

    Vehicle vehicle = {
        .base = {2, "Truck"},
        .speed = 80.0,
        .cargo_capacity = 1000
    };

    Enemy enemy = {
        .base = {3, "Goblin"},
        .level = 5,
        .xp = 50
    };

    /* Use base class functions via casting */
    printf("Using generic Entity functions:\n");
    entity_print(&player.base);
    entity_print(&vehicle.base);
    entity_print(&enemy.base);

    printf("\nUsing type-specific functions:\n");
    player_print(&player);
    vehicle_print(&vehicle);
    enemy_print(&enemy);

    /* Modify through base */
    printf("\nModifying through base:\n");
    entity_set_name(&player.base, "SuperHero");
    player_print(&player);

    /* Pointer casting (polymorphism simulation) */
    Entity *entities[] = {
        &player.base,
        &vehicle.base,
        &enemy.base
    };

    printf("\nPolymorphism via array of base pointers:\n");
    for (int i = 0; i < 3; i++) {
        entity_print(entities[i]);
    }

    /* Check layout */
    printf("\nMemory layout validation:\n");
    printf("  player base at: %p\n", (void*)&player);
    printf("  player.base at: %p\n", (void*)&player.base);
    printf("  Same address: %s\n",
           &player == (Player*)&player.base ? "YES" : "NO");

    return 0;
}
