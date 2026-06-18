{ pkgs, ... }:
{
  languages.rust = {
    enable = true;
    channel = "nightly";
    targets = [ "x86_64-unknown-linux-musl" ];
  };

  services.redis = {
    enable = true;
    package = pkgs.valkey;
  };

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

  env = {
    KANADE_SERVER__DB__URL = "postgresql://kanade:hello@127.0.0.1:5439/kanade";
    DATABASE_URL = "postgresql://kanade:hello@127.0.0.1:5439/kanade";
    KANADE_SERVER__VALKEY__URL = "redis://127.0.0.1:6379";
  };
}
