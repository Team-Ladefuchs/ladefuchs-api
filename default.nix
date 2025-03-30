{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell {
  buildInputs = [
    pkgs.sqlx-cli
    pkgs.podman-compose
  ];

  shellHook = ''
        	if ! podman ps | grep -q postgres; then
        		podman-compose --file $(pwd)/docker-compose/docker-compose.yml up -d
    			echo "Start postgres"
        	fi
            echo "Ready"
  '';
}
