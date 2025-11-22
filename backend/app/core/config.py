from pydantic_settings import BaseSettings

class Settings(BaseSettings):
    MONGO_URI: str
    MONGO_DB_NAME: str = "forcefocus"

    class Config:
        env_file = ".env"

settings = Settings()
