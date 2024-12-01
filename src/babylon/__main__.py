from dotenv import load_dotenv
from config import Config

def main():
    # Access configuration variables
    secret_key = Config.SECRET_KEY
    database_url = Config.DATABASE_URL

    print(f"Running with SECRET_KEY={secret_key}")
    print(f"Database URL: {database_url}")
    print(f"Debug mode: {Config.DEBUG}")

    # Your application logic...

if __name__ == "__main__":
    main()