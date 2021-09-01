# import config.
# You can change the default config with `make cnf="config_special.env" build`
cnf ?= config.env
include $(cnf)
export $(shell sed 's/=.*//' $(cnf))

# grep the version from VERSION.txt which is where the mix files take their version number from.
VERSION=`cat VERSION.txt | tr -d '\n'`

# HELP
# This will output the help for each task
# thanks to https://marmelab.com/blog/2016/02/29/auto-documented-makefile.html
.PHONY: help test

help: ## This help.
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

.DEFAULT_GOAL := help

db: ## Start the database via docker-compose using environment variables from `config.env`.
	docker-compose up -d
	
# There are no commands to stop/remove the database. Use regular docker-compose for that.

dev: ## Run service in development mode
	iex -S mix phx.server
	
test: ## Run tests
	mix test
	
test-watch: ## Run and watch tests
	mix test.watch --stale

build: ## Build the container
	docker build -t ladefuchs_server .
	
# TODO: push to registry
# release: build
# 	???

migrate: ## Run ecto migrations forward
	@mix ecto.migrate
	
rollback: ## Rollback latest ecto migration
	@mix ecto.rollback
