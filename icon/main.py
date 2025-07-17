from PIL import Image, ImageDraw, ImageFont

img = Image.new("RGBA", (512, 512), (255, 255, 255, 0))
draw = ImageDraw.Draw(img)

try:
    font = ImageFont.truetype("font/lmromancaps.otf", size=200)
except Exception as e:
    print("Failed to load font", e)
    exit(1)


texts = [
    # "Bib" - 蓝色
    {"pos": (210, 135), "text": "B", "fill": (0, 0, 150)},
    {"pos": (350, 165), "text": "i", "fill": (0, 0, 150)},  # 向上浮动
    {"pos": (400, 135), "text": "b", "fill": (0, 0, 150)},
    # "CiTe" - 红色
    {"pos": (150, 315), "text": "C", "fill": (180, 0, 0)},
    {"pos": (290, 345), "text": "i", "fill": (180, 0, 0)},  # 向下浮动
    {"pos": (320, 315), "text": "T", "fill": (180, 0, 0)},
    {"pos": (440, 345), "text": "e", "fill": (180, 0, 0)},  # 向下浮动
]

min_x, min_y, max_x, max_y = float("inf"), float("inf"), float("-inf"), float("-inf")
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

for t in texts:
    new_pos = (t["pos"][0] + x_offset, t["pos"][1] + y_offset)
    draw.text(new_pos, t["text"], fill=t["fill"], font=font)

img.save("../assets/logo.png", dpi=(500, 500))
print("图片已保存到../assets/logo.png")
