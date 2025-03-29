{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell {
  buildInputs = [
    pkgs.sqlx-cli
    pkgs.podman-compose
  ];

  shellHook = ''
    	if ! podman ps --quiet | grep -q postgres; then
    		podman-compose --file ./docker-compose/docker-compose.yml up -d > /dev/null 2>&1
    	fi
        echo "Ready"
  '';
}
