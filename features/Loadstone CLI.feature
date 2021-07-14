Feature: Loadstone CLI

Background:
    Given loadstone is configured for custom greeting "Welcome to Loadstone:"
    And loadstone is configured for a golden bank
    And loadstone is configured for serial recovery
    And just loadstone is loaded on the devkit

Scenario: Loadstone requests a golden recovery firmware image
    When the devkit is powered on
    Then the following is printed to the cli
    """

    Welcome to Loadstone:
    No current image.
    Attempting to restore from bank 2.
    Attempting to restore from golden bank 3.
    -- Loadstone Recovery Mode --
    Attempting golden image recovery to external flash...
    Please send golden firmware image via XMODEM.
    """
