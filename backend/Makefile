image:
	@mkdir -p build/target
	@mkdir -p build/registry
	@docker build -f dockerfiles/dev -t smart-fluid-flow-meter-backend-image .
.PHONY: image

image-prod: image
	@docker run --rm \
		-v $(PWD)/Cargo.lock:/smart-fluid-flow-meter/Cargo.lock \
		-v $(PWD)/Cargo.toml:/smart-fluid-flow-meter/Cargo.toml \
		-v $(PWD)/.env:/backend/.env \
		-v $(PWD)/.env.sample:/backend/.env.sample \
		-v $(PWD)/build/target:/smart-fluid-flow-meter/target \
		-v $(PWD)/src:/smart-fluid-flow-meter/src \
		-w /smart-fluid-flow-meter/ \
		smart-fluid-flow-meter-backend-image \
		cargo build --release
	@docker build -f dockerfiles/prod -t backend-prod .
.PHONY: image-prod

ssh: clean
	@docker run --rm -it \
		-p 3000:3000 \
		-v $(PWD)/Cargo.lock:/backend/Cargo.lock \
		-v $(PWD)/Cargo.toml:/backend/Cargo.toml \
		-v $(PWD)/.env:/backend/.env \
		-v $(PWD)/.env.sample:/backend/.env.sample \
		-v $(PWD)/Makefile:/backend/Makefile \
		-v $(PWD)/README.md:/backend/README.md \
		-v $(PWD)/build:/backend/build \
		-v $(PWD)/dockerfiles:/backend/dockerfiles \
		-v $(PWD)/src:/backend/src \
		-v $(PWD)/tests:/backend/tests \
		-v $(PWD)/dev-environments/vim/tmp:/root/.local/share/nvim \
		-w /backend/ \
		--name smart-fluid-flow-meter-backend-dev \
		smart-fluid-flow-meter-backend-image \
		bash
.PHONY: ssh

ssh-prod: image-prod
	@docker run --rm -it -p 3000:3000 \
		--name smart-fluid-flow-meter-backend-prod-ssh \
		smart-fluid-flow-meter-backend-image-prod \
		bash
.PHONY: ssh-prod

build: image
	@docker run --rm -it -p 3000:3000 \
		-v $(PWD)/Cargo.lock:/smart-fluid-flow-meter/Cargo.lock \
		-v $(PWD)/Cargo.toml:/smart-fluid-flow-meter/Cargo.toml \
		-v $(PWD)/.env:/backend/.env \
		-v $(PWD)/.env.sample:/backend/.env.sample \
		-v $(PWD)/build/registry:/usr/local/cargo/registry \
		-v $(PWD)/build/target:/smart-fluid-flow-meter/target \
		-v $(PWD)/src:/smart-fluid-flow-meter/src \
		-w /smart-fluid-flow-meter/ \
		--name smart-fluid-flow-meter-backend-dev \
		smart-fluid-flow-meter-backend-image \
		cargo build
.PHONY: build

start:
	@docker compose -f dockerfiles/docker-compose-firestore.yaml down --remove-orphans
	@docker compose -f dockerfiles/docker-compose-firestore.yaml up --build --abort-on-container-exit
.PHONY: start-mysql

start-prod: image-prod
	@docker run --rm -it -p 3000:3000 \
		-e GOOGLE_APPLICATION_CREDENTIALS=/service-account-key.json \
		--env-file .env \
		-v $(PWD)/service-account-key.json:/service-account-key.json \
		--name smart-fluid-flow-meter-backend-prod \
		smart-fluid-flow-meter-backend-image-prod \
		./smart_fluid_flow_meter_backend
.PHONY: start-prod

# Starts a container with a neovim development environment ready to use
vim: image
	@docker build -f dockerfiles/dev-vim -t smart-fluid-flow-meter-backend-vim-image .
	@docker run --rm -it \
		-v $(PWD)/Cargo.lock:/backend/Cargo.lock \
		-v $(PWD)/Cargo.toml:/backend/Cargo.toml \
		-v $(PWD)/.env:/backend/.env \
		-v $(PWD)/.env.sample:/backend/.env.sample \
		-v $(PWD)/Makefile:/backend/Makefile \
		-v $(PWD)/README.md:/backend/README.md \
		-v $(PWD)/build:/backend/build \
		-v $(PWD)/dockerfiles:/backend/dockerfiles \
		-v $(PWD)/src:/backend/src \
		-v $(PWD)/tests:/backend/tests \
		-v $(PWD)/dev-environments/vim/tmp:/root/.local/share/nvim \
		-w /backend/ \
		smart-fluid-flow-meter-backend-vim-image \
		sh -c "nvim"
.PHONY: vim

test:
	@docker compose -f dockerfiles/docker-compose-test.yaml down --remove-orphans
	@docker compose -f dockerfiles/docker-compose-test.yaml up --build \
		--abort-on-container-exit \
		--exit-code-from smart-fluid-flow-meter-backend
.PHONY: test

format:
	@docker run --rm -v $(PWD)/:/smart-fluid-flow-meter/ -w /smart-fluid-flow-meter/ smart-fluid-flow-meter-backend-image cargo fmt
.PHONY: format

check-format:
	@docker run --rm -v $(PWD)/:/smart-fluid-flow-meter/ -w /smart-fluid-flow-meter/ smart-fluid-flow-meter-backend-image cargo fmt --check
.PHONY: check-format

verify: clean build image-prod check-format test
.PHONY: verify

clean:
	-@docker kill smart-fluid-flow-meter-backend-dev 2>/dev/null ||:
	-@docker rm smart-fluid-flow-meter-backend-dev 2>/dev/null ||:
	-@docker kill smart-fluid-flow-meter-backend-prod 2>/dev/null ||:
	-@docker rm smart-fluid-flow-meter-backend-prod 2>/dev/null ||:
.PHONY: clean
