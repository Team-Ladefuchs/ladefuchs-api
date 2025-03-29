{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell {
  buildInputs = [
    pkgs.podman-compose
  ];

  shellHook = ''
    sudo podman-compose --file ./docker-compose/docker-compose.yml up -d > /dev/null 2>&1
    echo "Ready"
  '';
}
