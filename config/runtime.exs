import Config

config :yactatt,
  cta_bustracker_key: System.get_env("CTA_BUSTRACKER_KEY"),
  cta_traintracker_key: System.get_env("CTA_TRAINTRACKER_KEY")
