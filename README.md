# Dengbot

Silly meme Slack bot. Watches the channel defined in the configuration for the keyphrase "deng" and grants points to the Slack user that posted it if it's their first use of the keyphrase of the day, and extra points if it is the first use of the keyphrase in the last 24 hours.

Users can also request the scoreboard by sending the Slack command defined in the bot configuration. It will query the database and return a formatted scoreboard message to Slack.
### Prerequisites
You'll need the Rust compiler (stable), Docker and an up-to-date install of OpenSSL to build. Cargo should handle the rest!
### Building

1. Complete the `.env` file with your database URL, Slack tokens and port information. All entries are necessary - it will not compile if any are missing!

2. Build the Docker container:

```
docker build <path>
```

## Deployment

Deploy the Docker container to the host of your choice.

Your network setup and security rules will need to allow incoming connections to the port specified in your `.env`, outgoing connections to your database and incoming/outgoing  to Slack API URLs.

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details
