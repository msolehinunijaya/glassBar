#!/bin/sh
# music.wav = the synthesized bed with the music paused for the adhan (26.0-33.7 s) and the
# first phrase of the app's own adhan (src/assets/adhan.ogg) placed at 26.65 s. Run from promo/.
ffmpeg -y -hide_banner -loglevel error -i audio/music_bed.wav -i ../src/assets/adhan.ogg -filter_complex \
"[0]volume='if(lt(t,26.0),1,if(lt(t,26.45),1-(t-26)/0.45,if(lt(t,33.62),0,if(lt(t,33.76),(t-33.62)/0.14,1))))':eval=frame[m];\
[1]atrim=0.85:7.9,asetpts=PTS-STARTPTS,aresample=48000,aformat=channel_layouts=stereo,afade=t=in:d=0.15,afade=t=out:st=5.6:d=1.45,volume=0.42,adelay=26650|26650,apad[a];\
[m]aresample=48000,aformat=channel_layouts=stereo[m2];[m2][a]amix=inputs=2:normalize=0:duration=first" audio/music.wav
