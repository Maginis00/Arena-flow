#!/bin/bash
# Regenerates the placeholder sound effects in this folder with ffmpeg.
# They are synthesised from noise and sine sweeps, so they are our own work
# (CC0, no third-party licence). Swap in any CC0 sample with the same file
# name to replace one; the game only knows the file names (src/fx/sounds.rs).
set -eu
cd "$(dirname "$0")"

R=44100
# sweep F0 F1 DUR: phase of a sine gliding linearly from F0 to F1 Hz.
sweep() { echo "2*PI*($1*t+($2-$1)*t*t/(2*$3))"; }

make() { # name duration expression [extra filter]
  ffmpeg -hide_banner -loglevel error -y \
    -f lavfi -i "aevalsrc='$3':s=$R:d=$2" \
    -af "${4:-anull},afade=t=out:st=$(awk "BEGIN{print $2*0.7}"):d=$(awk "BEGIN{print $2*0.3}"),volume=0.8" \
    -ac 1 -c:a libvorbis -q:a 4 "$1.ogg"
}

# Noise source in aevalsrc: random(0) is uniform in [0,1).
N="(2*random(0)-1)"

make shot_hitscan 0.09 "0.6*$N*exp(-t*60)+0.5*sin($(sweep 2200 500 0.09))*exp(-t*40)" "highpass=f=900"
make shot_projectile 0.14 "0.7*sin($(sweep 700 160 0.14))*exp(-t*22)+0.25*$N*exp(-t*45)" "lowpass=f=4000"
make swing 0.2 "$N*sin(PI*t/0.2)" "bandpass=f=1400:width_type=h:w=1200,volume=2.5"
make enemy_hit 0.05 "$N*exp(-t*90)" "bandpass=f=3000:width_type=h:w=2500,volume=2"
make enemy_die 0.22 "0.8*sin($(sweep 340 55 0.22))*exp(-t*12)+0.5*$N*exp(-t*25)" "lowpass=f=2500,acrusher=bits=8:mix=0.4"
make player_hit 0.28 "0.9*sin($(sweep 150 45 0.28))*exp(-t*9)+0.6*$N*exp(-t*18)" "lowpass=f=1200,acrusher=bits=6:mix=0.5,volume=1.4"
make player_die 0.9 "0.8*sin($(sweep 420 35 0.9))*exp(-t*3)+0.4*$N*exp(-t*5)" "lowpass=f=1800,aecho=0.8:0.6:120:0.4"
make charge_tell 0.4 "(0.6*sin($(sweep 280 950 0.4))+0.25*sin(2*$(sweep 280 950 0.4)))*min(1,t*30)"
make pickup 0.3 "0.5*sin(2*PI*880*t)*exp(-t*25)*lt(t,0.1)+0.5*sin(2*PI*1320*(t-0.08))*exp(-(t-0.08)*14)*gte(t,0.08)"
make wave_clear 0.7 "0.4*sin(2*PI*523*t)*exp(-t*8)+0.4*sin(2*PI*659*(t-0.12))*exp(-(t-0.12)*7)*gte(t,0.12)+0.45*sin(2*PI*784*(t-0.24))*exp(-(t-0.24)*5)*gte(t,0.24)" "aecho=0.7:0.5:90:0.3,volume=3"
