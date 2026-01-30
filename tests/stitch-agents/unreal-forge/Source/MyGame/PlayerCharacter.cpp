/**
 * Stitch test fixture for Forge (Unreal Engine C++ agent)
 */

#include "PlayerCharacter.h"
#include "Camera/CameraComponent.h"
#include "GameFramework/SpringArmComponent.h"
#include "Components/InputComponent.h"

APlayerCharacter::APlayerCharacter()
{
    PrimaryActorTick.bCanEverTick = true;
    
    // TODO: Intentional issue - magic numbers
    MoveSpeed = 600.0f;
    RotationSpeed = 100.0f;
    UnusedCounter = 0;
    
    // TODO: Intentional issue - no null check after CreateDefaultSubobject
    SpringArmComponent = CreateDefaultSubobject<USpringArmComponent>(TEXT("SpringArm"));
    SpringArmComponent->SetupAttachment(RootComponent);
    SpringArmComponent->TargetArmLength = 300.0f;
    
    CameraComponent = CreateDefaultSubobject<UCameraComponent>(TEXT("Camera"));
    CameraComponent->SetupAttachment(SpringArmComponent);
}

void APlayerCharacter::BeginPlay()
{
    Super::BeginPlay();
    
    // TODO: Intentional issue - UE_LOG without proper category
    UE_LOG(LogTemp, Warning, TEXT("PlayerCharacter BeginPlay"));
}

void APlayerCharacter::Tick(float DeltaTime)
{
    Super::Tick(DeltaTime);
    
    HandleMovement(DeltaTime);
    HandleRotation(DeltaTime);
}

void APlayerCharacter::SetupPlayerInputComponent(UInputComponent* PlayerInputComponent)
{
    Super::SetupPlayerInputComponent(PlayerInputComponent);
    
    // TODO: Intentional issue - deprecated input binding style
    PlayerInputComponent->BindAxis("MoveForward", this, &APlayerCharacter::HandleMovement);
}

void APlayerCharacter::HandleMovement(float DeltaTime)
{
    // TODO: Intentional issue - unused local variable
    float unusedSpeed = 500.0f;
    
    FVector Movement = FVector::ZeroVector;
    ProcessInput(Movement);
    
    AddMovementInput(Movement, MoveSpeed * DeltaTime);
}

void APlayerCharacter::HandleRotation(float DeltaTime)
{
    // TODO: Intentional issue - empty function body
}

void APlayerCharacter::ProcessInput(FVector& OutMovement)
{
    // TODO: Intentional issue - direct GetWorld() call without null check
    APlayerController* PC = GetWorld()->GetFirstPlayerController();
    
    if (PC)
    {
        FVector Forward = PC->GetControlRotation().Vector();
        OutMovement = Forward;
    }
}
