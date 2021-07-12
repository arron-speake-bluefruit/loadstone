Feature: Loadstone CLI

#need to find a way of defining what BUILD of loadstone I am using (mainly 1 and 2 from sprint 6)

Background: 
    Given the custom greeting of Loadstone has been set to "Welcome to Loadstone:"

#no images at all with golden bank configured
#no images at all with no golden bank configured
Rule: Loadstone reports that it has entered recovery mode

    Scenario: Loadstone requests a golden, recovery firmware image 
        Given an STM32F412G connected via USART
        And Loadstone has been configured with a golden bank and serial recovery mode
        And only Loadstone is loaded on the dev kit
        When the dev kit is powered on
        Then the following is printed to the CLI:
        """
        Welcome to Loadstone:
        Checking for image updates...
        No current image.
        Attempting to restore from bank 2.
        Attempting to restore from bank 4.
        Attempting to restore from golden bank 3.
        -- Loadstone Recovery Mode --
        Attempting golden image recovery to external flash...
        Please send golden firmware image via XMODEM.
        """

    Scenario: Loadstone requests a recovery firmware image
        Given an STM32F412G connected via USART
        And Loadstone has been configured with serial recovery mode and no golden bank
        And only Loadstone is loaded on the dev kit
        When the dev kit is powered on
        Then the following is printed to the CLI:
        """
        Welcome to Loadstone:
        Checking for image updates...
        No current image.
        Attempting to restore from bank 2.
        Attempting to restore from bank 3.
        Attempting to restore from bank 4.
        -- Loadstone Recovery Mode --
        Attempting image recovery to MCU flash...
        Please send firmware image via XMODEM.
        """

#non golden image uploaded to golden recovery mode
#golden image uploaded to golden recovery mode
#non golden image uploaded to a non golden recovery mode
Rule: Loadstone reports the result of uploading a recovery firmware image

    Scenario: A golden firmware image is uploaded when a golden bank configured Loadstone is in recovery mode
        Given an STM32F412G connected via USART
        And Loadstone has been configured with a golden bank and serial recovery mode
        And Loadstone is in serial recovery mode
        When a golden firmware image is uploaded using XMODEM
        Then the following is printed to the CLI:
        """
        Finished flashing golden image.
        Rebooting..
        """

    Scenario: A non-golden firmware image is uploaded when a golden bank configured Loadstone is in recovery mode
        Given an STM32F412G connected via USART
        And Loadstone has been configured with a golden bank and serial recovery mode
        And Loadstone is in serial recovery mode
        When a non-golden firmware image is uploaded using XMODEM
        Then the following is printed to the CLI:
        """
        FATAL: Flashed image is not a golden image.
        FATAL: Image did not flash correctly.
        [Logic Error] -> Image is not golden
        Rebooting..
        """


    Scenario Outline: A firmware image is uploaded when Loadstone is in recovery mode
        Given an STM32F412G connected via USART
        And Loadstone has been configured with serial recovery mode and no golden bank
        When a <golden> firmware image is uploaded using XMODEM
        Then the following is printed to the CLI:
        """
        Finished flashing image.
        Rebooting..
        """

    Examples:
        |golden|
        |golden|
        |non-golden|


#image in boot bank and identical image in internal bank (no update)
#image in boot bank but not internal bank or golden bank or external bank
Rule: Loadstone reports that no update occured and the boot firmware image was loaded

Background:
    Given Loadstone has been configured to manage bank 1 internal bootable 100kB, bank 2 internal 100kB, bank 3 external golden 100kB, bank 4 external 100kB

    Scenario: A back up firmware image prevents Loadstone from updating the boot firmawre image
        Given an STM32F412G connected via USART
        And a valid firmware image is in the boot bank
        And an indentical firmware image is in bank 1
        When the dev kit is powered on
        Then the following is printed to the CLI:
        """
        Welcome to Loadstone:
        Checking for image updates...
        [stm32f4 flash (Internal)] Scanning bank 2 for a newer image...
        Attempting to boot from default bank.
        """

    Scenario: A solitary boot firmware image is loaded by Loadstone
        Given an STM32F412G 


#image in bank 2 but not boot bank or golden bank or external bank
#image external bank but not boot bank internal bank or golden bank
#image in boot bank and a different image in internal bank (update)
Rule: Loadstone reports that an update occured then loads the updated firmware image


#image in golden bank but not in boot bank, internal bank or external bank
Rule: Loadstone reports when a golden firmware image has been copied to the boot bank and loaded

Background:
    Given Loadstone has been configured to manage bank 1 internal bootable 100kB, bank 2 internal 100kB, bank 3 external golden 100kB, bank 4 external 100kB

Scenario: golden image copied from golden bank
Scenario: non golden image copied from golden bank
