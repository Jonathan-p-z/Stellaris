# Stellaris

![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)

Simulateur galactique navigable en temps réel, généré procéduralement en Rust/raylib.  
Développé en solo pour une game jam — thème : simulation d'étoiles et de galaxies.

![screenshot](assets/README.png)

## À propos

Stellaris génère procéduralement trois galaxies spirales navigables en temps réel. Chaque galaxie est construite à partir de bras logarithmiques perturbés par du bruit de Perlin, avec un noyau dense O/B et une distribution spectrale IMF réaliste. L'espace intergalactique est peuplé d'événements dynamiques — supernovae, pulsars, quasars, étoiles filantes — qui s'animent en continu.

Le rendu repose sur un pipeline en deux passes : la scène 3D est capturée dans une render texture, puis un shader bloom post-process GLSL est appliqué sur le framebuffer entier avant affichage. Un système de LOD dynamique combine billboards texturés pour les objets proches et draw_point3D pour les objets lointains.

## Modes de rendu

Un menu de sélection s'affiche au démarrage :

| Mode | Étoiles | Bloom | Cible |
|------|---------|-------|-------|
| HIGH | 75 600 | Shader GLSL post-process | GPU dédié recommandé |
| LOW  | 39 000 | Désactivé | iGPU / Core i5 / Ryzen 5 / 8 Go RAM |

## Fonctionnalités

**Génération procédurale**
- 3 galaxies spirales avec bras logarithmiques, bruit de Perlin et inclinaisons distinctes
- 75 600 étoiles (mode HIGH) / 39 000 (mode LOW) avec types spectraux O/B/A/F/G/K/M et distribution IMF approximée
- Couleur dominante par galaxie : bleutée (jeune, O/B), orangée (vieille, G/K), rougeâtre (irrégulière, K/M)
- Noyau dense surreprésenté en O/B, généré indépendamment des bras

**Rendu**
- Pipeline 2 passes : scène → render texture → shader bloom GLSL (kernel gaussien 3×3)
- LOD dynamique : billboards texturés < 60u, draw_point3D au-delà, culling > 450u
- Nébuleuses intégrées aux bras galactiques, 60 billboards par nébuleuse avec variation de teinte per-cloud
- Trous noirs avec disque d'accrétion (blanc chaud / orange-rouge) et lensing gravitationnel
- Poussière cosmique : 55 000 points en sphère 360° + disque local aplati, culling > 300u
- Textures générées procéduralement au démarrage (falloff gaussien) via le crate `image`

**Événements temps réel**
- Supernovae pulsantes, pulsars clignotants, quasars à triple halo
- Amas globulaires (40–80 étoiles), étoiles binaires
- Étoiles filantes avec traînée (pool de 8, spawn aléatoire)

**Navigation**
- 6DOF libre dans l'espace 3D
- 3 paliers de vitesse (exploration / traversée / saut)
- 60 FPS stable en mode LOW sur i5/Ryzen 5, 8 Go RAM, sans GPU dédié

## Contrôles

| Touche | Action |
|--------|--------|
| Z / S | Avancer / Reculer |
| Q / D | Strafe gauche / droite |
| Espace | Monter |
| C | Descendre |
| Souris (clic droit) | Orienter la vue |
| Molette | Changer de vitesse (3 paliers) |
| Échap | Quitter |

## Compilation

**Prérequis :** Rust stable (`rustup`), `cmake` (requis par raylib-sys)

```bash
# Linux
cargo build --release
./target/release/stellaris

# Windows
cargo build --release
target\release\stellaris.exe
```

Les textures (`assets/star_halo.png`, `assets/nebula_cloud.png`) sont générées automatiquement au premier lancement si absentes. Aucune dépendance externe à installer manuellement.

## Stack technique

| Crate | Rôle |
|-------|------|
| `raylib-rs 5.5` | Rendu, fenêtre, inputs |
| `noise` | Bruit de Perlin pour les bras spiraux |
| `rand` | Distributions aléatoires reproductibles |
| `image` | Génération des textures PNG au démarrage |

## Intelligence artificielle

Ce projet a été développé avec l'assistance de Claude (Anthropic).  
Claude a contribué à la documentation, à certaines idées de fonctionnalités.  
Tout le code a été écrit par le développeur.  
Conformément aux règles de la game jam, ce projet porte le tag **IA**.

## Licence

[GPLv3](LICENSE)
