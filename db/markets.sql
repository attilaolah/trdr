DROP DATABASE IF EXISTS markets;
CREATE DATABASE markets;

\c markets

-- CoinMarketCap is currently the source of all of this data.

-- API fetch operations.
-- Each API call must be logged in this table.
CREATE TABLE updates (
    id SERIAL PRIMARY KEY,

    -- URL, including the query string.
    -- E.g. "https://pro-api.coinmarketcap.com/v1/fiat/map&include_metals=true".
    -- HTTPS is optional to allow using a reverse proxy running in the same cluster.
    url TEXT NOT NULL CHECK (url ~ '^https?://'),

    -- Values below are returned by the API.

    -- An internal error code for the current error.
    -- If a unique platform error code is not available the HTTP status code is returned.
    error_code INTEGER,
    -- An error message to go along with the error code.
    error_message TEXT CHECK (error_message <> ''),
    -- Number of API call credits that were used for this call.
    credit_count INTEGER NOT NULL CHECK (credit_count >= 0),
    -- Current timestamp on the server.
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    -- Amount of time taken to generate this response.
    elapsed INTERVAL NOT NULL CHECK (elapsed >= interval '0'),

    -- Notice from the server (undocumented).
    notice TEXT CHECK (notice <> '')
);

-- API endpoint: /v1/fiat/map
CREATE TABLE fiats (
    -- The unique CoinMarketCap ID for this asset.
    id INTEGER PRIMARY KEY,
    -- The name of this asset.
    name TEXT NOT NULL CHECK (name <> ''),
    -- The currency sign for this asset.
    sign TEXT NOT NULL CHECK (sign <> ''),
    -- The ticker symbol for this asset, always in all caps.
    symbol VARCHAR(3) UNIQUE NOT NULL CHECK (symbol ~ '^[A-Z]{3}$'),

    -- Last update operation to this value.
    -- The operation may not have changed any values.
    last_update INTEGER NOT NULL REFERENCES updates(id)
);

CREATE TYPE metal_unit AS ENUM (
    'ounce',
    'troy_ounce'
);

-- API endpoint: /v1/fiat/map include_metals=true
CREATE TABLE metals (
    -- The unique CoinMarketCap ID for this asset.
    id INTEGER PRIMARY KEY,
    -- The name of this asset, without the unit.
    name TEXT NOT NULL CHECK (name <> ''),
    -- The ticker symbol (code) for this asset, always in all caps (undocumented).
    code VARCHAR(3) UNIQUE NOT NULL CHECK (code ~ '^[A-Z]{3}$'),

    -- Unit (exctracted from the name).
    unit metal_unit NOT NULL,

    -- Last update operation to this value.
    -- The operation may not have changed any values.
    last_update INTEGER NOT NULL REFERENCES updates(id)
);

CREATE TYPE tracking_status AS ENUM ('active', 'inactive', 'untracked');

-- API endpoint: /v1/cryptocurrencies/map
CREATE TABLE cryptocurrencies (
    -- The unique cryptocurrency ID for this cryptocurrency.
    id INTEGER PRIMARY KEY,
    -- The name of this cryptocurrency.
    name TEXT NOT NULL CHECK (name <> ''),
    -- The ticker symbol for this cryptocurrency, NOT always in all caps.
    -- NOTE: CoinMarketCap claims this is always in all-caps, but it really isn't.
    symbol TEXT NOT NULL CHECK (symbol <> ''),
    -- The web URL friendly shorthand version of this cryptocurrency name.
    slug TEXT NOT NULL CHECK (slug ~ '^[0-9a-z-]+$'),
    -- Whether this cryptocurrency has at least 1 active market currently being tracked by the platform.
    is_active BOOLEAN NOT NULL,
    -- The listing status of the cryptocurrency.
    status tracking_status NOT NULL,
    -- Timestamp of the date this cryptocurrency was first available on the platform.
    first_historical_data TIMESTAMP WITH TIME ZONE,
    -- Timestamp of the last time this cryptocurrency's market data was updated.
    last_historical_data TIMESTAMP WITH TIME ZONE,

    -- Metadata about the parent cryptocurrency platform this cryptocurrency belongs to if it is a token.
    platform INTEGER REFERENCES cryptocurrencies(id) CHECK (
        (platform IS NULL) = (platform_token IS NULL)
    ),
    -- The token address on the parent platform cryptocurrency.
    platform_token TEXT CHECK (platform_token <> ''),

    -- Last update operation to this value.
    -- The operation may not have changed any values.
    last_update INTEGER NOT NULL REFERENCES updates(id)
);

-- API endpoint: /v1/exchange/map 
CREATE TABLE exchanges (
    -- The unique CoinMarketCap ID for this exchange.
    id INTEGER PRIMARY KEY,
    -- The name of this exchange.
    name TEXT NOT NULL CHECK (name <> ''),
    -- The web URL friendly shorthand version of this exchange name.
    slug TEXT NOT NULL UNIQUE CHECK (slug ~ '^[0-9a-z-]+$'),
    -- Whether this exchange is still being actively tracked and updated.
    is_active BOOLEAN NOT NULL,
    -- The listing status of the exchange.
    status tracking_status NOT NULL,
    -- Timestamp of the earliest market data record available to query using our historical endpoints.
    -- NULL if there is no historical data currently available for this exchange.
    first_historical_data TIMESTAMP WITH TIME ZONE,
    -- Timestamp of the latest market data record available to query using our historical endpoints.
    -- null if there is no historical data currently available for this exchange.
    last_historical_data TIMESTAMP WITH TIME ZONE,

    -- Last update operation to this value.
    -- The operation may not have changed any values.
    last_update INTEGER NOT NULL REFERENCES updates(id)
);

-- API endpoint: /v1/exchanges/info
CREATE TABLE exchange_infos (
    -- The unique CoinMarketCap ID for this exchange.
    id INTEGER PRIMARY KEY REFERENCES exchanges (id),
    -- Link to a CoinMarketCap hosted logo png for this exchange. 64px is default size returned.
    -- Replace "64x64" in the image path with these alternative sizes: 16, 32, 64, 128, 200.
    logo TEXT NOT NULL CHECK (logo ~ '^https://.+64x64.+'),
    -- A CoinMarketCap supplied brief description of this cryptocurrency exchange.
    description TEXT CHECK (description <> ''),
    -- Launch date for this exchange.
    date_launched DATE NOT NULL,
    -- A Markdown formatted message outlining a condition that is impacting
    -- the availability of the exchange's market data or the secure use of the exchange.
    notice TEXT CHECK (notice <> ''),
    -- Maker fee on this exchange.
    maker_fee NUMERIC(6,4) CHECK (maker_fee >= 0),
    -- Taker fee on this exchange.
    taker_fee NUMERIC(6,4) CHECK (taker_fee >= 0),
    -- The number of weekly visitors.
    weekly_visits INTEGER CHECK (weekly_visits >= 0),
    -- Reported all time spot volume in the specified currency.
    spot_volume_usd NUMERIC(20,8) CHECK (spot_volume_usd >= 0),
    -- Reported last update time of the spot volume.
    spot_volume_last_updated TIMESTAMP WITH TIME ZONE,

    -- Last update operation to this value.
    -- The operation may not have changed any values.
    last_update INTEGER NOT NULL REFERENCES updates(id)
);
CREATE TABLE exchange_fiats (
    id INTEGER NOT NULL REFERENCES exchanges(id),
    symbol VARCHAR(3) NOT NULL REFERENCES fiats(symbol),
    PRIMARY KEY (id, symbol)
);
CREATE TYPE exchange_url_kind AS ENUM (
    'website',
    'twitter',
    'blog',
    'chat',
    'fee'
);
CREATE TABLE exchange_urls (
    id INTEGER NOT NULL REFERENCES exchanges(id),
    kind exchange_url_kind NOT NULL,
    url TEXT NOT NULL CHECK (url ~ '^https://')
);
