from PIL import Image, ImageDraw, ImageFont


def create_gradient_background(width, height):
    gradient = Image.new("RGBA", (width, height))
    gradient_draw = ImageDraw.Draw(gradient)

    # 创建现代化的径向渐变背景
    center_x, center_y = width // 2, height // 2
    max_distance = ((width/2)**2 + (height/2)**2)**0.5

    for y in range(height):
        for x in range(width):
            # 计算到中心的距离
            distance = ((x - center_x)**2 + (y - center_y)**2)**0.5
            ratio = min(distance / max_distance, 1.0)

            # 创建从中心的浅蓝白色到边缘的深蓝紫色的径向渐变
            # 中心颜色：浅蓝白色 (240, 245, 255)
            # 边缘颜色：深蓝紫色 (45, 55, 120)
            r = int(240 - (240 - 45) * ratio)
            g = int(245 - (245 - 55) * ratio)
            b = int(255 - (255 - 120) * ratio)

            # 添加微妙的噪点效果增加质感
            import random
            random.seed(x * 1000 + y)  # 确保可重复的随机性
            noise = random.randint(-8, 8)
            r = max(0, min(255, r + noise))
            g = max(0, min(255, g + noise))
            b = max(0, min(255, b + noise))

            gradient_draw.point((x, y), fill=(r, g, b, 255))

    return gradient


def create_rounded_rectangle(width, height, radius):
    background = Image.new("RGBA", (width, height), (0, 0, 0, 0))
    bg_draw = ImageDraw.Draw(background)

    bg_draw.rounded_rectangle(
        [(0, 0), (width, height)], radius=radius, fill=(255, 255, 255, 255)
    )

    return background


try:
    font = ImageFont.truetype("font/lmromancaps.otf", size=150)
except Exception as e:
    print("Failed to load font", e)
    exit(1)


texts = [
    # "Bib"
    {"pos": (260, 195), "text": "B", "fill": (0, 0, 150), "shadow": (0, 0, 100)},
    {"pos": (367, 195), "text": "i", "fill": (0, 0, 150), "shadow": (0, 0, 100)},
    {"pos": (410, 195), "text": "b", "fill": (0, 0, 150), "shadow": (0, 0, 100)},
    # "CiTeX"
    {"pos": (150, 320), "text": "C", "fill": (180, 0, 0), "shadow": (120, 0, 0)},
    {"pos": (260, 320), "text": "i", "fill": (180, 0, 0), "shadow": (120, 0, 0)},
    {"pos": (300, 320), "text": "T", "fill": (180, 0, 0), "shadow": (120, 0, 0)},
    {"pos": (390, 355), "text": "E", "fill": (180, 0, 0), "shadow": (120, 0, 0)},
    {"pos": (470, 320), "text": "X", "fill": (180, 0, 0), "shadow": (120, 0, 0)},
]


def _centralize(texts, draw, img):
    min_x, min_y, max_x, max_y = (
        float("inf"),
        float("inf"),
        float("-inf"),
        float("-inf"),
    )
    for t in texts:
        bbox = draw.textbbox(t["pos"], t["text"], font=font)
        min_x = min(min_x, bbox[0])
        min_y = min(min_y, bbox[1])
        max_x = max(max_x, bbox[2])
        max_y = max(max_y, bbox[3])

    text_width = max_x - min_x
    text_height = max_y - min_y

    x_offset = (img.width - text_width) / 2 - min_x
    y_offset = (img.height - text_height) / 2 - min_y
    return x_offset, y_offset


def logo():
    img = Image.new("RGBA", (512, 512), (0, 0, 0, 100))
    draw = ImageDraw.Draw(img)
    background = create_rounded_rectangle(512, 512, 60)

    gradient = create_gradient_background(512, 512)
    background.paste(gradient, (0, 0), background)
    img.paste(background, (0, 0))
    x_offset, y_offset = _centralize(texts, draw, img)
    for t in texts:
        new_pos = (t["pos"][0] + x_offset, t["pos"][1] + y_offset)

        for offset in [(1, 1), (-1, -1), (1, -1), (-1, 1)]:
            glow_pos = (new_pos[0] + offset[0], new_pos[1] + offset[1])
            draw.text(glow_pos, t["text"], fill=(*t["fill"], 30), font=font)

    for t in texts:
        new_pos = (t["pos"][0] + x_offset, t["pos"][1] + y_offset)

        shadow_pos = (new_pos[0] + 3, new_pos[1] + 3)
        draw.text(shadow_pos, t["text"], fill=(*t["shadow"], 80), font=font)

    for t in texts:
        new_pos = (t["pos"][0] + x_offset, t["pos"][1] + y_offset)
        draw.text(new_pos, t["text"], fill=t["fill"], font=font)
    img.save("../assets/logo.png", dpi=(500, 500))
    print("图片已保存到../assets/logo.png")


def transparent_logo():
    img = Image.new("RGBA", (512, 512), (255, 255, 255, 0))
    draw = ImageDraw.Draw(img)

    x_offset, y_offset = _centralize(texts, draw, img)

    for t in texts:
        new_pos = (t["pos"][0] + x_offset, t["pos"][1] + y_offset)
        draw.text(new_pos, t["text"], fill=t["fill"], font=font)

    img.save("../assets/transparent_logo.png", dpi=(500, 500))
    print("图片已保存到../assets/transparent_logo.png")
