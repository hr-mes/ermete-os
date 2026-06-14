# Inietta Nix nel PATH dell'utente asincronamente se il demone è attivo
if [ -e /var/opt/nix/profiles/default/etc/profile.d/nix-daemon.sh ]; then
    . /var/opt/nix/profiles/default/etc/profile.d/nix-daemon.sh
fi
