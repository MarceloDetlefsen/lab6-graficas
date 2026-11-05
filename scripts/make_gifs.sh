#!/usr/bin/env bash
set -e

BIN=./target/release/lab6
F=captures/auto/frames
mkdir -p "$F" docs/gifs

# Une frames PNG en un GIF liviano (400px de ancho, 15 fps)
make_gif () {  # $1 = prefijo de frames, $2 = salida
  ffmpeg -y -loglevel error -framerate 15 -i "$F/$1_%03d.png" \
    -vf "scale=400:-1:flags=lanczos,split[a][b];[a]palettegen[p];[b][p]paletteuse" \
    "$2"
}

# --- Estrella: hervor, 60 frames, time 0 -> 6 ---
for i in $(seq 0 59); do
  t=$(awk "BEGIN{print $i*0.1}")
  $BIN --shader 1 --view color --light -0.45,-0.56,-0.70 --time $t \
    --out $F/star_$(printf %03d $i).png
done
make_gif star docs/gifs/estrella.gif

# --- Zebes: nubes, 60 frames, time 0 -> 42 ---
for i in $(seq 0 59); do
  t=$(awk "BEGIN{print $i*0.7}")
  $BIN --shader 2 --view color --light -0.45,-0.56,-0.70 --time $t \
    --out $F/clouds_$(printf %03d $i).png
done
make_gif clouds docs/gifs/zebes_nubes.gif

# --- Zebes: luz rotando de frente a trasera (terminador, lava, especular, bump) ---
for i in $(seq 0 59); do
  read lx lz < <(awk "BEGIN{a=$i*2.5*3.14159/180; printf \"%.3f %.3f\", -0.9*sin(a), -0.9*cos(a)}")
  $BIN --shader 2 --view color --light $lx,-0.3,$lz --time 0.0 \
    --out $F/light_$(printf %03d $i).png
done
make_gif light docs/gifs/zebes_luz.gif

# --- Adrian: flujo y vórtice, 60 frames, time 0 -> 30 ---
for i in $(seq 0 59); do
  t=$(awk "BEGIN{print $i*0.5}")
  $BIN --shader 3 --view color --light -0.45,-0.56,-0.70 --time $t \
    --out $F/adrian_$(printf %03d $i).png
done
make_gif adrian docs/gifs/adrian.gif

# --- Portada: los tres cuerpos lado a lado; Zebes usa la animación de luz ---
ffmpeg -y -loglevel error -framerate 15 -i $F/star_%03d.png \
  -framerate 15 -i $F/light_%03d.png -framerate 15 -i $F/adrian_%03d.png \
  -filter_complex "[0]scale=300:-1[a];[1]scale=300:-1[b];[2]scale=300:-1[c];[a][b][c]hstack=3,split[x][y];[x]palettegen[p];[y][p]paletteuse" \
  docs/gifs/demo.gif

ls -lh docs/gifs/
