set -e; set -o pipefail


USER=conner
HOME=/home/conner
NAME=zip.conner.SearchCuts

# Seems need to install search provider globally :-(
# - https://gitlab.gnome.org/GNOME/gnome-shell/-/issues/3060
cp $NAME.search-provider.ini \
  /usr/share/gnome-shell/search-providers/

# Need a valid .desktop app to work
# Installing it globally to keep in sync with above
cp $NAME.desktop /usr/share/applications/

install -d -o $USER "$HOME/.config/systemd/user"
install -o $USER $NAME.service "$HOME/.config/systemd/user/$NAME.service"


