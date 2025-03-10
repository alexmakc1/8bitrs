from PIL import Image, ImageDraw

def create_sprite(name, size, draw_func):
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    draw_func(draw)
    img.save(f'assets/sprites/{name}.png')

# Player sprite (simple 8-bit character)
def draw_player(draw):
    # Body
    draw.rectangle([4, 4, 11, 11], fill=(200, 150, 100))
    # Head
    draw.rectangle([5, 1, 10, 4], fill=(255, 200, 150))
    # Arms
    draw.rectangle([2, 5, 4, 9], fill=(200, 150, 100))
    draw.rectangle([11, 5, 13, 9], fill=(200, 150, 100))
    # Legs
    draw.rectangle([4, 11, 6, 14], fill=(50, 50, 150))
    draw.rectangle([9, 11, 11, 14], fill=(50, 50, 150))

# Tree sprite
def draw_tree(draw):
    # Trunk
    draw.rectangle([6, 8, 10, 15], fill=(139, 69, 19))
    # Leaves
    draw.rectangle([4, 2, 12, 8], fill=(34, 139, 34))
    draw.rectangle([2, 4, 14, 6], fill=(34, 139, 34))

# Tree stump sprite
def draw_tree_stump(draw):
    # Stump
    draw.rectangle([6, 8, 10, 12], fill=(139, 69, 19))
    # Top
    draw.ellipse([5, 6, 11, 8], fill=(160, 82, 45))

# Wall sprite
def draw_wall(draw):
    # Stone texture
    for i in range(0, 16, 4):
        for j in range(0, 16, 4):
            draw.rectangle([i, j, i+3, j+3], fill=(120, 120, 120))
            draw.rectangle([i+1, j+1, i+2, j+2], fill=(100, 100, 100))

# Goblin sprite
def draw_goblin(draw):
    # Body
    draw.rectangle([4, 4, 11, 11], fill=(50, 150, 50))
    # Head
    draw.rectangle([5, 1, 10, 4], fill=(100, 200, 100))
    # Arms
    draw.rectangle([2, 5, 4, 9], fill=(50, 150, 50))
    draw.rectangle([11, 5, 13, 9], fill=(50, 150, 50))
    # Legs
    draw.rectangle([4, 11, 6, 14], fill=(50, 150, 50))
    draw.rectangle([9, 11, 11, 14], fill=(50, 150, 50))

# Fire sprite
def draw_fire(draw):
    # Base
    draw.rectangle([4, 12, 12, 15], fill=(50, 50, 50))
    # Flames
    draw.polygon([(6, 4), (10, 4), (12, 8), (8, 6), (4, 8)], fill=(255, 100, 0))
    draw.polygon([(7, 2), (9, 2), (11, 6), (8, 4), (5, 6)], fill=(255, 200, 0))

# Fishing spot sprite
def draw_fishing_spot(draw):
    # Water ripples
    draw.ellipse([2, 2, 14, 14], fill=(0, 100, 255, 128))
    draw.ellipse([4, 4, 12, 12], fill=(0, 150, 255, 128))
    draw.ellipse([6, 6, 10, 10], fill=(100, 200, 255, 128))

# Sword sprite
def draw_sword(draw):
    # Blade
    draw.polygon([(8, 2), (10, 2), (10, 12), (8, 12)], fill=(200, 200, 200))
    # Handle
    draw.rectangle([7, 12, 11, 14], fill=(139, 69, 19))
    # Guard
    draw.rectangle([6, 11, 12, 12], fill=(255, 215, 0))

# Axe sprite
def draw_axe(draw):
    # Handle
    draw.rectangle([7, 4, 9, 14], fill=(139, 69, 19))
    # Head
    draw.polygon([(4, 2), (12, 2), (12, 6), (4, 6)], fill=(200, 200, 200))

# Logs sprite
def draw_logs(draw):
    # Log
    draw.rectangle([4, 6, 12, 10], fill=(139, 69, 19))
    # End grain
    draw.ellipse([3, 5, 13, 11], fill=(160, 82, 45))
    draw.ellipse([11, 5, 13, 11], fill=(160, 82, 45))

# Fish sprite
def draw_fish(draw):
    # Body
    draw.ellipse([4, 6, 12, 10], fill=(100, 100, 255))
    # Tail
    draw.polygon([(2, 8), (4, 6), (4, 10)], fill=(100, 100, 255))

# Water sprite
def draw_water(draw):
    # Base water color
    draw.rectangle([0, 0, 15, 15], fill=(0, 100, 255, 200))
    # Wave details
    for i in range(0, 16, 4):
        draw.arc([i, 2, i+8, 6], 0, 180, fill=(100, 200, 255, 128), width=1)
        draw.arc([i-2, 6, i+6, 10], 0, 180, fill=(100, 200, 255, 128), width=1)
        draw.arc([i+2, 10, i+10, 14], 0, 180, fill=(100, 200, 255, 128), width=1)

# Road sprite
def draw_road(draw):
    # Base path
    draw.rectangle([0, 0, 15, 15], fill=(150, 140, 130))
    # Gravel details
    for i in range(0, 16, 4):
        for j in range(0, 16, 4):
            draw.point([i+1, j+1], fill=(130, 120, 110))
            draw.point([i+2, j+2], fill=(170, 160, 150))

# Fence sprite
def draw_fence(draw):
    # Vertical posts
    draw.rectangle([2, 4, 4, 12], fill=(139, 69, 19))
    draw.rectangle([12, 4, 14, 12], fill=(139, 69, 19))
    # Horizontal boards
    draw.rectangle([2, 6, 14, 8], fill=(160, 82, 45))
    draw.rectangle([2, 10, 14, 12], fill=(160, 82, 45))

# Castle wall sprite
def draw_castle_wall(draw):
    # Main wall
    draw.rectangle([0, 4, 15, 15], fill=(180, 180, 180))
    # Crenellations
    for i in range(0, 16, 4):
        draw.rectangle([i, 0, i+2, 4], fill=(180, 180, 180))
    # Stone details
    for i in range(0, 16, 4):
        for j in range(4, 16, 4):
            draw.rectangle([i+1, j+1, i+3, j+3], fill=(150, 150, 150))

# Castle door sprite
def draw_castle_door(draw):
    # Door frame
    draw.rectangle([2, 0, 13, 15], fill=(139, 69, 19))
    # Door arch
    draw.arc([2, -6, 13, 4], 0, 180, fill=(160, 82, 45), width=2)
    # Door details
    draw.rectangle([4, 2, 11, 14], fill=(120, 60, 15))
    draw.ellipse([9, 7, 10, 8], fill=(255, 215, 0))  # Door handle

# Castle stairs sprite
def draw_castle_stairs(draw):
    # Steps
    for i in range(4):
        draw.rectangle([0, 12-i*3, 15-i*4, 15-i*3], fill=(180, 180, 180))
        draw.rectangle([1, 13-i*3, 14-i*4, 14-i*3], fill=(150, 150, 150))

# Bridge sprite
def draw_bridge(draw):
    # Main planks
    draw.rectangle([0, 6, 15, 10], fill=(139, 69, 19))
    # Side rails
    draw.rectangle([0, 4, 15, 6], fill=(160, 82, 45))
    draw.rectangle([0, 10, 15, 12], fill=(160, 82, 45))
    # Support posts
    draw.rectangle([2, 2, 4, 14], fill=(120, 60, 15))
    draw.rectangle([12, 2, 14, 14], fill=(120, 60, 15))

# Path sprite
def draw_path(draw):
    # Base dirt color
    draw.rectangle([0, 0, 15, 15], fill=(170, 140, 100))
    # Path details
    for i in range(0, 16, 2):
        for j in range(0, 16, 2):
            if (i + j) % 4 == 0:
                draw.point([i, j], fill=(150, 120, 80))
            else:
                draw.point([i, j], fill=(190, 160, 120))

# Create all sprites
sprites = [
    ('player', 16, draw_player),
    ('tree', 16, draw_tree),
    ('tree_stump', 16, draw_tree_stump),
    ('wall', 16, draw_wall),
    ('goblin', 16, draw_goblin),
    ('fire', 16, draw_fire),
    ('fishing_spot', 16, draw_fishing_spot),
    ('sword', 16, draw_sword),
    ('axe', 16, draw_axe),
    ('logs', 16, draw_logs),
    ('fish', 16, draw_fish),
    ('water', 16, draw_water),
    ('road', 16, draw_road),
    ('fence', 16, draw_fence),
    ('castle_wall', 16, draw_castle_wall),
    ('castle_door', 16, draw_castle_door),
    ('castle_stairs', 16, draw_castle_stairs),
    ('bridge', 16, draw_bridge),
    ('path', 16, draw_path),
]

for name, size, draw_func in sprites:
    create_sprite(name, size, draw_func) 