# Shaders procedurales en un software renderer

Entrega del laboratorio de Gráficas por Computadora. El proyecto renderiza una
misma esfera `.obj` en CPU, sin GPU ni texturas externas. La apariencia de cada
cuerpo se genera en los fragment shaders con ruido OpenSimplex/Fbm, normales,
posición, tiempo y luz.

## Requisitos

- Rust y Cargo instalados.
- El proyecto se ejecuta con el renderer de CPU incluido; no requiere una GPU.

## Compilación y ejecución

Para abrir el modo interactivo:

```bash
cargo run --release
```

También se puede renderizar un único frame sin abrir la ventana. El modo headless
recibe el shader (`1`, `2` o `3`), la vista, la luz, el tiempo y la ruta de
salida. Por ejemplo:

```bash
cargo run --offline --release -- \
  --shader 2 \
  --view color \
  --light -0.45,-0.56,-0.70 \
  --time 0.0 \
  --out captures/auto/zebes.png
```

Las vistas válidas son `height`/`altura`, `biome`/`bioma`, `cracks`/`grietas` y
`color`. El PNG se escribe en la ruta indicada y el proceso termina.

## Controles

| Tecla | Acción |
|---|---|
| `1` | Selecciona la estrella. |
| `2` | Selecciona Zebes. |
| `3` | Selecciona Adrian. |
| `B` | Modo de triángulos rellenos con barycentrics. |
| `W` | Modo wireframe. |
| `V` | Cicla las vistas de Rocky: altura, bioma, grietas y color. |
| `D` | Activa/desactiva la vista global de FBM en escala de grises. |
| `[` / `]` | Disminuye/aumenta la frecuencia de la vista `D`. |
| `J` / `L` | Rota la dirección de la luz hacia la izquierda/derecha. |
| `P` | Guarda una captura incremental en `captures/shot_NNN.png`. |

Las flechas también rotan la cámara y la rueda del mouse ajusta el zoom.

## Cuerpos celestes

### Estrella violeta/magenta

**Nombre:** Estrella violeta/magenta (`Shader::Star`).

![Estrella violeta/magenta](docs/captures/estrella.png)

**Captura:** `docs/captures/estrella.png`.

El shader está en `src/shaders/star.rs`. Es una superficie emisiva: no usa
`light_dir`. `temperature` combina el oscurecimiento hacia el limbo mediante
`surface_mu`, granulación animada con `noise::fbm4` y manchas de baja frecuencia.
`shade` convierte esa temperatura en una paleta que pasa suavemente por índigo,
violeta, magenta y blanco rosado; después suma la corona y un pulso global.

| Constante | Qué cambia visualmente |
|---|---|
| `GRANULATION_FREQ` | Escala espacial del grano superficial. |
| `GRANULATION_SPEED` | Velocidad del hervor de la granulación. |
| `GRANULATION_AMP` | Contraste del grano. |
| `SPOT_FREQ` | Tamaño y distribución de las manchas. |
| `SPOT_SPEED` | Velocidad independiente de las manchas. |
| `SPOT_THRESHOLD` | Cantidad de superficie que entra en las manchas. |
| `LIMB_EXPONENT` | Fuerza del oscurecimiento del borde. |
| `CORONA_EXPONENT` | Qué tan concentrado queda el halo en el borde. |
| `CORONA_INTENSITY` | Brillo de la corona rosado-magenta. |
| `PULSE_AMP` / `PULSE_SPEED` | Amplitud y velocidad del pulso de brillo. |

La paleta se define con `PALETTE_WHITE_PINK`, `PALETTE_MAGENTA`,
`PALETTE_VIOLET` y `PALETTE_INDIGO`.

### Zebes, planeta rocoso

![Zebes con luz frontal](docs/captures/zebes_frontal.png)

**Capturas:** `docs/captures/zebes_frontal.png` y
`docs/captures/zebes_noche.png`.

El shader está en `src/shaders/rocky.rs`. `height_field` genera roca cálida con
`ridged` y domain warp; `biome_mask` y `crack_mask` separan pequeños acentos de
Brinstar y grietas. `surface_color` combina la paleta ámbar con esas máscaras.
En la fase de iluminación, `bumped_normal` calcula un gradiente por diferencias
finitas, `lit_surface_color` aplica difusa y el especular del mar, y `shade`
agrega lava nocturna y nubes.

| Constante | Qué cambia visualmente |
|---|---|
| `VEIN_STRETCH` | Longitud y dirección dominante de las vetas. |
| `VEIN_ANGLE` | Rotación diagonal del eje de estiramiento. |
| `WARP_AMP` / `WARP_FREQ` | Curvatura y escala del domain warp. |
| `RIDGE_SHARPNESS` | Finura y contraste de las crestas. |
| `BIOME_THRESHOLD` | Cobertura del bioma verde. |
| `CRACK_THRESHOLD` | Cantidad de grietas de lava. |
| `SEA_LEVEL` | Cobertura del mar ácido. |
| `BUMP_STRENGTH` / `BUMP_DELTA` | Intensidad y escala del relieve. |
| `SEA_SPEC_POWER` / `SEA_SPEC_STRENGTH` | Tamaño e intensidad del reflejo del mar. |
| `LAVA_GLOW` | Brillo emisivo de lava en el lado nocturno. |
| `CLOUD_SPEED` / `CLOUD_OPACITY` | Movimiento y opacidad de las nubes. |

### Adrian, mundo denso inspirado en *Project Hail Mary*

![Adrian en tiempo 0](docs/captures/adrian_t000.png)

**Capturas:** `docs/captures/adrian_t000.png`,
`docs/captures/adrian_t030.png` y `docs/captures/adrian_lateral.png`.

El shader vive en `src/shaders/gas_giant.rs`, aunque la interpretación visual de
la entrega es Adrian como un mundo denso, no como un gigante gaseoso literal.
`domain_warp` aplica tres pasadas de warp y `band_pattern_at` mezcla la
latitud distorsionada con FBM para producir marmoleado líquido. `vortex_domain`
integra el giro del vórtice con todo el campo, mientras `vortex_mask` crea un
borde orgánico. `surface_color` mezcla la paleta verde con el parche
naranja-rojizo y `shade` aplica luz, limbo y rim atmosférico.

| Constante | Qué cambia visualmente |
|---|---|
| `BAND_FREQ` | Cantidad de bandas principales. |
| `WARP1_AMP` / `WARP2_AMP` / `WARP3_AMP` | Intensidad de la turbulencia líquida. |
| `WARP1_FREQ` / `WARP2_FREQ` / `WARP3_FREQ` | Escala de cada detalle del warp. |
| `WARP1_SPEED` / `WARP2_SPEED` | Velocidad independiente del flujo. |
| `VORTEX_CENTER` | Posición del parche naranja. |
| `VORTEX_RADIUS` | Tamaño del parche de tormenta. |
| `VORTEX_INFLUENCE_RADIUS` | Alcance de la curvatura de las bandas vecinas. |
| `VORTEX_STRENGTH` | Intensidad del enrollamiento. |
| `VORTEX_EDGE_NOISE` | Irregularidad del borde del vórtice. |
| `VORTEX_SPIN` | Velocidad de rotación del vórtice. |

## Efectos evaluables

| Efecto | Cuerpo | Archivo y función | Evidencia | Parámetro modificable |
|---|---|---|---|---|
| Material animado | Estrella | `src/shaders/star.rs`, `temperature` y `shade` | `docs/captures/estrella.png` <!-- TODO: GIF --> | `GRANULATION_SPEED`: acelera el hervor. |
| Material animado | Adrian | `src/shaders/gas_giant.rs`, `domain_warp`, `band_pattern_at` y `shade` | `docs/captures/adrian_t000.png`, `docs/captures/adrian_t030.png` <!-- TODO: GIF --> | `WARP1_SPEED` y `WARP2_SPEED`: cambian el flujo. |
| Respuestas diferentes a la luz | Zebes | `src/shaders/rocky.rs`, `lit_surface_color`; iluminación común en `src/shaders.rs`, `diffuse_factor` | `docs/captures/zebes_frontal.png` | `SEA_SPEC_POWER`: hace el reflejo marino más amplio o concentrado. |
| Día y noche | Zebes | `src/shaders/rocky.rs`, `lit_surface_color` y `shade` | `docs/captures/zebes_frontal.png`, `docs/captures/zebes_noche.png` <!-- TODO: GIF --> | `LAVA_GLOW`: aumenta la emisión de las grietas en sombra. |
| Nubes procedurales | Zebes | `src/shaders/rocky.rs`, `cloud_mask`, `lit_cloud_color` y `shade` | `docs/captures/zebes_nubes_t000.png`, `docs/captures/zebes_nubes_t020.png` <!-- TODO: GIF --> | `CLOUD_SPEED`: cambia el desplazamiento temporal de las nubes. |
| Bump mapping | Zebes | `src/shaders/rocky.rs`, `bumped_normal` | `docs/captures/zebes_frontal.png` | `BUMP_STRENGTH`: refuerza o suaviza el relieve. |
| Tormenta/vórtice | Adrian | `src/shaders/gas_giant.rs`, `vortex_domain` y `vortex_mask` | `docs/captures/adrian_t000.png`, `docs/captures/adrian_lateral.png` | `VORTEX_RADIUS` y `VORTEX_STRENGTH`: controlan tamaño y giro. |

## Limitaciones conocidas

- La esfera proporcionada tiene 98 vértices y una silueta facetada; los
  shaders no modifican la malla ni agregan modelos.
- El renderer es de CPU, por lo que aumentar demasiado las octavas de ruido
  reduce el framerate.
- Las capturas de `captures/auto/` son artefactos de trabajo y no forman parte
  de la evidencia versionada; las evidencias estables están bajo `docs/`.

## Uso de LLMs como apoyo

Se utilizó un LLM como apoyo para inspeccionar la estructura del renderer,
proponer y revisar código, y documentar el proceso. El código fue revisado y
entendido para esta entrega; puedo modificar directamente las constantes y las
funciones descritas para ajustar la apariencia y el rendimiento.
