/**
 * Stitch test fixture for Forge (Unreal Engine C++ agent)
 * 
 * This file contains intentional issues for testing remediation:
 * - Missing UPROPERTY macros
 * - Raw pointers without null checks
 * - Coding standard violations
 */

#pragma once

#include "CoreMinimal.h"
#include "GameFramework/Character.h"
#include "PlayerCharacter.generated.h"

UCLASS()
class MYGAME_API APlayerCharacter : public ACharacter
{
    GENERATED_BODY()

public:
    APlayerCharacter();

protected:
    virtual void BeginPlay() override;
    virtual void Tick(float DeltaTime) override;
    virtual void SetupPlayerInputComponent(class UInputComponent* PlayerInputComponent) override;

private:
    // TODO: Intentional issue - missing UPROPERTY macro
    float MoveSpeed;
    float RotationSpeed;
    
    // TODO: Intentional issue - raw pointer without UPROPERTY
    class UCameraComponent* CameraComponent;
    class USpringArmComponent* SpringArmComponent;
    
    // TODO: Intentional issue - unused member variable
    int32 UnusedCounter;
    
    void HandleMovement(float DeltaTime);
    void HandleRotation(float DeltaTime);
    
    // TODO: Intentional issue - non-const reference parameter
    void ProcessInput(FVector& OutMovement);
};
