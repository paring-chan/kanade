{ pkgs, ... }:
{
  languages.rust.enable = true;

  packages = with pkgs; [
    openssl
    sqlx-cli
  ];

  services.postgres = {
    enable = true;
    listen_addresses = "127.0.0.1";
    port = 5439;

    initialDatabases = [
      {
        name = "kanade";
        user = "kanade";
        pass = "hello";
      }
    ];
  };

  env.KANADE_SERVER_DB__URL = "postgresql://kanade:hello@127.0.0.1:5439/kanade";
}
