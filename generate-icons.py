#!/usr/bin/env python3
import os
import subprocess
import shutil
from PIL import Image

base_dir = "/Volumes/CBR DATA/项目/news-browser"
artwork_svg = os.path.join(base_dir, "logo-artwork.svg")
base_plate = "/Volumes/CBR DATA/下载/a58ph-bw0yw.png"
icons_dir = os.path.join(base_dir, "src-tauri", "icons")
work_dir = os.path.join(base_dir, "icon-work")

# Clean work dir
if os.path.exists(work_dir):
    shutil.rmtree(work_dir)
os.makedirs(work_dir)

def sips_resize(size, out_path, source=artwork_svg):
    subprocess.run([
        "sips", "-z", str(size), str(size), source, "--out", out_path, "-s", "format", "png"
    ], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

def make_icon(size):
    """把 logo 图案合成到用户提供的 macOS 底板图上"""
    # 底板缩放到目标尺寸
    base = Image.open(base_plate).convert("RGBA")
    base = base.resize((size, size), Image.LANCZOS)

    # 渲染 artwork（透明背景）到目标尺寸
    artwork_png = os.path.join(work_dir, f"artwork_{size}.png")
    sips_resize(size, artwork_png, artwork_svg)
    artwork = Image.open(artwork_png).convert("RGBA")

    # 合成
    icon = Image.alpha_composite(base, artwork)
    return icon

# Generate referenced PNGs
png_sizes = {
    "32x32.png": 32,
    "128x128.png": 128,
    "128x128@2x.png": 256,
    "256x256.png": 256,
}
for name, size in png_sizes.items():
    icon = make_icon(size)
    icon.save(os.path.join(icons_dir, name), format="PNG")

# Generate iconset for macOS .icns
iconset_dir = os.path.join(work_dir, "logo.iconset")
os.makedirs(iconset_dir)

icns_sizes = [
    (16, "icon_16x16.png"),
    (32, "icon_16x16@2x.png"),
    (32, "icon_32x32.png"),
    (64, "icon_32x32@2x.png"),
    (128, "icon_128x128.png"),
    (256, "icon_128x128@2x.png"),
    (256, "icon_256x256.png"),
    (512, "icon_256x256@2x.png"),
    (512, "icon_512x512.png"),
    (1024, "icon_512x512@2x.png"),
]
for size, filename in icns_sizes:
    icon = make_icon(size)
    icon.save(os.path.join(iconset_dir, filename), format="PNG")

subprocess.run([
    "iconutil", "-c", "icns", iconset_dir, "-o", os.path.join(icons_dir, "icon.icns")
], check=True)

# Generate Windows .ico from 256x256 composite
ico_source = make_icon(256)
ico_source.save(
    os.path.join(icons_dir, "icon.ico"),
    format="ICO",
    sizes=[(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
)

# Cleanup
shutil.rmtree(work_dir)
print("Icons generated successfully.")
