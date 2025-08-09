#!/bin/bash

# A quick and dirty script which 
# re-encodes streaming_shape_tools extracted SSS files into mp4 video files.
# A better thing would probably be doing it natively.

VIDEOS_DIR="$PWD/videos"

[[ ! -d "$VIDEOS_DIR" ]] && mkdir -pv $VIDEOS_DIR

for f in $(find . -mindepth 1 -maxdepth 1 -type d | sort -V | sed '/videos/d' ); do
    pushd $f
        name=$(printf $f | sed 's/.\///g' | sed 's/_extracted//g')
        echo "Video To Build $name $PWD";

        [[ -f "list.txt" ]] && rm list.txt;
        for frame in $(find frames/ -mindepth 1 -maxdepth 1 -type f | sort -V); do
            echo "file '$PWD/$frame'" >> list.txt
        done

        ffmpeg -y -safe 0 -f concat  -i list.txt -framerate 15 -vf "settb=AVTB,setpts=N/15/TB,fps=15" -c:v libx264 -preset veryslow -b:v 128k -an $VIDEOS_DIR/$name.mp4
    popd
done
