DROP DATABASE IF EXISTS markets;
CREATE DATABASE markets;

\c markets

-- CoinMarketCap is currently the source of all of this data.

-- API fetch operations.
-- Each API call must be logged in this table.
CREATE TABLE updates (
    id SERIAL PRIMARY KEY,

    -- URL, including the query string.
    -- E.g. "https://sandbox-api.coinmarketcap.com/v1/fiat/map&include_metals=true".
    url TEXT NOT NULL,

    -- Values below are returned by the API.

    -- An internal error code for the current error.
    -- If a unique platform error code is not available the HTTP status code is returned.
    error_code INTEGER,
    -- An error message to go along with the error code.
    error_message TEXT,
    -- Number of API call credits that were used for this call.
    credit_count INTEGER NOT NULL,
    -- Current timestamp on the server.
    timestamp TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    -- Amount of time taken to generate this response.
    elapsed INTERVAL NOT NULL,

    -- Notice from the server (undocumented).
    notice TEXT
);

-- API endpoint: /v1/fiat/map
CREATE TABLE fiats (
    -- The unique CoinMarketCap ID for this asset.
    id INTEGER PRIMARY KEY,
    -- The name of this asset.
    name TEXT NOT NULL,
    -- The currency sign for this asset.
    sign TEXT,
    -- The ticker symbol for this asset, always in all caps.
    symbol VARCHAR(3) UNIQUE NOT NULL,

    -- Last update operation to this value.
    -- The operation may not have changed any values.
    last_update INTEGER NOT NULL,

    -- Validation:
    CONSTRAINT valid_symbol CHECK (symbol ~ '^[A-Z]{3}$'),

    -- Foreign keys:
    CONSTRAINT fk_last_update FOREIGN KEY (last_update) REFERENCES updates(id)
);

CREATE TYPE metal_unit AS ENUM ('ounce');

-- API endpoint: /v1/fiat/map include_metals=true
CREATE TABLE metals (
    -- The unique CoinMarketCap ID for this asset.
    id INTEGER PRIMARY KEY,
    -- The name of this asset, without the unit.
    name TEXT NOT NULL,
    -- The ticker symbol (code) for this asset, always in all caps (undocumented).
    code VARCHAR(3) UNIQUE NOT NULL,

    -- Unit (exctracted from the name).
    unit metal_unit NOT NULL,

    -- Last update operation to this value.
    -- The operation may not have changed any values.
    last_update INTEGER NOT NULL,

    -- Validation:
    CONSTRAINT valid_code CHECK (code ~ '^[A-Z]{3}$'),

    -- Foreign keys:
    CONSTRAINT fk_last_update FOREIGN KEY (last_update) REFERENCES updates(id)
);

CREATE TYPE tracking_status AS ENUM ('active', 'inactive', 'untracked');

-- API endpoint: /v1/cryptocurrencies/map
CREATE TABLE cryptocurrencies (
    -- The unique cryptocurrency ID for this cryptocurrency.
    id INTEGER PRIMARY KEY,
    -- The name of this cryptocurrency.
    name TEXT NOT NULL,
    -- The ticker symbol for this cryptocurrency, NOT always in all caps.
    -- NOTE: CoinMarketCap claims this is always in all-caps, but it really isn't.
    symbol TEXT NOT NULL,
    -- The web URL friendly shorthand version of this cryptocurrency name.
    slug TEXT NOT NULL,
    -- Whether this cryptocurrency has at least 1 active market currently being tracked by the platform.
    is_active BOOLEAN NOT NULL,
    -- The listing status of the cryptocurrency.
    status tracking_status NOT NULL,
    -- Timestamp of the date this cryptocurrency was first available on the platform.
    first_historical_data TIMESTAMP WITHOUT TIME ZONE,
    -- Timestamp of the last time this cryptocurrency's market data was updated.
    last_historical_data TIMESTAMP WITHOUT TIME ZONE,

    -- Metadata about the parent cryptocurrency platform this cryptocurrency belongs to if it is a token.
    platform INTEGER,
    -- The token address on the parent platform cryptocurrency.
    platform_token TEXT,

    -- CoinMarketCap ranking (undocumented).
    rank INTEGER NOT NULL,

    -- Last update operation to this value.
    -- The operation may not have changed any values.
    last_update INTEGER NOT NULL,

    -- Foreign keys:
    CONSTRAINT fk_last_update FOREIGN KEY (last_update) REFERENCES updates(id),
    CONSTRAINT fk_platform FOREIGN KEY (platform) REFERENCES cryptocurrencies(id)
);

-- API endpoint: /v1/exchange/map 
CREATE TABLE exchanges (
    -- The unique CoinMarketCap ID for this exchange.
    id INTEGER PRIMARY KEY,
    -- The name of this exchange.
    name TEXT NOT NULL,
    -- The web URL friendly shorthand version of this exchange name.
    slug TEXT NOT NULL UNIQUE,
    -- Whether this exchange is still being actively tracked and updated.
    is_active BOOLEAN NOT NULL,
    -- The listing status of the exchange.
    status tracking_status NOT NULL,
    -- Timestamp of the earliest market data record available to query using our historical endpoints.
    -- NULL if there is no historical data currently available for this exchange.
    first_historical_data TIMESTAMP WITHOUT TIME ZONE,
    -- Timestamp of the latest market data record available to query using our historical endpoints.
    -- null if there is no historical data currently available for this exchange.
    last_historical_data TIMESTAMP WITHOUT TIME ZONE,

    -- Last update operation to this value.
    -- The operation may not have changed any values.
    last_update INTEGER NOT NULL,

    -- Foreign keys:
    CONSTRAINT fk_last_update FOREIGN KEY (last_update) REFERENCES updates(id)
);

-- API endpoint: /v1/exchanges/info
CREATE TABLE exchange_infos (
    -- The unique CoinMarketCap ID for this exchange.
    id INTEGER PRIMARY KEY REFERENCES exchanges (id),
    -- Link to a CoinMarketCap hosted logo png for this exchange. 64px is default size returned.
    -- Replace "64x64" in the image path with these alternative sizes: 16, 32, 64, 128, 200.
    logo TEXT NOT NULL,
    -- A CoinMarketCap supplied brief description of this cryptocurrency exchange.
    description TEXT,
    -- Launch date for this exchange.
    date_launched DATE NOT NULL,
    -- A Markdown formatted message outlining a condition that is impacting
    -- the availability of the exchange's market data or the secure use of the exchange.
    notice TEXT,
    -- Maker fee on this exchange.
    maker_fee NUMERIC(6,4),
    -- Taker fee on this exchange.
    taker_fee NUMERIC(6,4),
    -- The number of weekly visitors.
    weekly_visits INTEGER,
    -- Reported all time spot volume in the specified currency.
    spot_volume_usd NUMERIC(20,8),
    -- Reported last update time of the spot volume.
    spot_volume_last_updated TIMESTAMP WITHOUT TIME ZONE,

    -- Last update operation to this value.
    -- The operation may not have changed any values.
    last_update INTEGER NOT NULL,

    -- Foreign keys:
    CONSTRAINT fk_last_update FOREIGN KEY (last_update) REFERENCES updates(id)
);
CREATE TABLE exchange_fiats (
    id INTEGER REFERENCES exchanges(id),
    symbol VARCHAR(3) REFERENCES fiats(symbol),
    PRIMARY KEY (id, symbol)
);
CREATE TABLE exchange_urls (
    id INTEGER REFERENCES exchanges(id),
    website TEXT[] NOT NULL,
    twitter TEXT[] NOT NULL,
    blog TEXT[] NOT NULL,
    chat TEXT[] NOT NULL,
    fee TEXT[] NOT NULL
);
