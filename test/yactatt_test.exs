defmodule YactattTest do
  use ExUnit.Case
  doctest Yactatt

  test "greets the world" do
    assert Yactatt.hello() == :world
  end
end
