# MIT License
#
# Copyright (c) 2021 Box ID Systems GmbH
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.
#
# created by Mathias Peter <mpneuried> & Stefan Fochler <istefo> at Box ID Systems GmbH
ARG ELIXIR_VERSION=1.12.2
ARG OTP_VERSION=24.0.5
ARG ALPINE_VERSION=3.14.0

FROM hexpm/elixir:${ELIXIR_VERSION}-erlang-${OTP_VERSION}-alpine-${ALPINE_VERSION} AS builder

ENV MIX_ENV=prod

# By convention, /opt is typically used for applications
WORKDIR /opt/app

# This step installs all the build tools we'll need
RUN apk add --no-cache build-base && \
  mix local.rebar --force && \
  mix local.hex --force

# Copy mix file to better cache dependencies between builds
COPY VERSION.txt .
COPY mix.exs .
COPY mix.lock .
RUN mix do deps.get, deps.compile

# This copies our app source code into the build container
COPY . .
RUN mix compile

RUN mix release ladefuchs_server && \
  mv _build/${MIX_ENV}/rel/ladefuchs_server /opt/built

# From this line onwards, we're in a new image, which will be the image used in production.
FROM alpine:${ALPINE_VERSION}

RUN apk add --no-cache openssl-dev ncurses-libs libstdc++

ENV REPLACE_OS_VARS=true

WORKDIR /opt/app

COPY --from=builder /opt/built .

CMD trap 'exit' INT; \
  /opt/app/bin/ladefuchs_server eval "Ladefuchs.ReleaseTasks.migrate()"; \
  /opt/app/bin/ladefuchs_server start
