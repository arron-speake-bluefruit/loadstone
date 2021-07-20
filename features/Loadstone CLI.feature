Feature: Loadstone CLI

Background:
    Given loadstone is configured for custom greeting "Welcome to Loadstone:"

Scenario: Loadstone requests a golden recovery firmware image
    Given loadstone is configured for a golden bank
    And loadstone is configured for serial recovery
    And just loadstone is loaded on the devkit
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

Scenario: Loadstone requests a recovery firmware image
    Given loadstone is not configured for a golden bank
    Given loadstone is configured for serial recovery
    And just loadstone is loaded on the devkit
    When the devkit is powered on
    Then the following is printed to the cli
    """

    Welcome to Loadstone:
    No current image.
    Attempting to restore from bank 2.
    Attempting to restore from bank 3.
    -- Loadstone Recovery Mode --
    Attempting image recovery to MCU flash...
    Please send firmware image via XMODEM.
    """

Scenario: A golden firmware image is uploaded when a golden bank configured loadstone is in recovery mode
    Given loadstone is configured for a golden bank
    And loadstone is configured for serial recovery
    And just loadstone is loaded on the devkit
    When the devkit is powered on
    And loadstone enters serial recovery mode
    And a golden firmware image is uploaded using xmodem
    Then the following is printed to the cli
    """
    Finished flashing golden image.
    Rebooting..
    """
