#!/usr/bin/env python3

import leglight
import click

host = "elgato-keylight.localdomain"
myLight = leglight.LegLight(host, 9123)


@click.group()
def cli():
    pass


@cli.command("on")
def turn_on():
    print("turning light on")
    myLight.brightness(4)
    myLight.on()


@cli.command("off")
def turn_off():
    print("turning light off")
    myLight.off()


if __name__ == "__main__":
    cli()
