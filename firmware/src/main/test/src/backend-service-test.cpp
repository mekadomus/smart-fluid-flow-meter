#define CATCH_CONFIG_MAIN
#include <catch.hpp>

#include "backend-service.hpp"

TEST_CASE("Destructor") {
  SECTION("ssid and password are reset after destructing instance") {
    BackendService* bs =
        BackendService::get_instance("ssid_one", "password_one");
    REQUIRE(bs->get_ssid() == "ssid_one");
    delete bs;
    bs = BackendService::get_instance("ssid_two", "password_two");
    REQUIRE(bs->get_ssid() == "ssid_two");
  }
}
